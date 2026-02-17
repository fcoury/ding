// ding-notifier — Standalone macOS notification helper using UNUserNotificationCenter.
//
// Reads a JSON request from stdin, delivers a notification, and writes a JSON
// response to stdout. Designed to be embedded in a .app bundle so that macOS
// grants notification permission to the bundle identifier.
//
// Build:
//   clang -fobjc-arc -O2 -mmacosx-version-min=11.0 \
//     -framework UserNotifications -framework Foundation \
//     -o ding-notifier notifier.m

#import <Foundation/Foundation.h>
#import <UserNotifications/UserNotifications.h>

// ---------------------------------------------------------------------------
// Delegate — handles foreground presentation and user interaction
// ---------------------------------------------------------------------------

@interface DingDelegate : NSObject <UNUserNotificationCenterDelegate>
@property (nonatomic, copy) void (^onResponse)(NSString *status);
@end

@implementation DingDelegate

// Show the banner even when this app is in the "foreground".
- (void)userNotificationCenter:(UNUserNotificationCenter *)center
       willPresentNotification:(UNNotification *)notification
         withCompletionHandler:(void (^)(UNNotificationPresentationOptions))handler {
    handler(UNNotificationPresentationOptionBanner | UNNotificationPresentationOptionSound);
}

// Capture the user's response (click or dismiss).
- (void)userNotificationCenter:(UNUserNotificationCenter *)center
didReceiveNotificationResponse:(UNNotificationResponse *)response
         withCompletionHandler:(void (^)(void))handler {
    NSString *action = response.actionIdentifier;
    if ([action isEqualToString:UNNotificationDefaultActionIdentifier]) {
        if (self.onResponse) self.onResponse(@"clicked");
    } else if ([action isEqualToString:UNNotificationDismissActionIdentifier]) {
        if (self.onResponse) self.onResponse(@"closed");
    } else {
        if (self.onResponse) self.onResponse(action);
    }
    handler();
}

@end

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

// Write a JSON response to stdout and exit.
static void respond(NSString *status, NSString *error) {
    NSMutableDictionary *dict = [NSMutableDictionary dictionary];
    dict[@"status"] = status;
    if (error) dict[@"error"] = error;
    NSData *json = [NSJSONSerialization dataWithJSONObject:dict options:0 error:nil];
    NSFileHandle *out = [NSFileHandle fileHandleWithStandardOutput];
    [out writeData:json];
    [out writeData:[@"\n" dataUsingEncoding:NSUTF8StringEncoding]];
}

static void respondAndExit(NSString *status, NSString *error) {
    respond(status, error);
    exit(error ? 1 : 0);
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

int main(int argc, const char *argv[]) {
    @autoreleasepool {
        // Read all of stdin into a JSON dictionary.
        NSFileHandle *input = [NSFileHandle fileHandleWithStandardInput];
        NSData *data = [input readDataToEndOfFile];
        if (!data || data.length == 0) {
            respondAndExit(@"error", @"empty stdin");
        }

        NSError *parseErr = nil;
        NSDictionary *req = [NSJSONSerialization JSONObjectWithData:data options:0 error:&parseErr];
        if (!req) {
            respondAndExit(@"error", [NSString stringWithFormat:@"invalid JSON: %@",
                                      parseErr.localizedDescription]);
        }

        NSString *title   = req[@"title"]   ?: @"Notification";
        NSString *message = req[@"message"] ?: @"";
        NSString *subtitle = req[@"subtitle"];
        NSString *sound   = req[@"sound"];
        BOOL waitForClick = [req[@"wait_for_click"] boolValue];

        UNUserNotificationCenter *center = [UNUserNotificationCenter currentNotificationCenter];
        if (!center) {
            respondAndExit(@"error", @"UNUserNotificationCenter unavailable");
        }

        // Set up delegate before anything else so we can receive responses.
        DingDelegate *delegate = [[DingDelegate alloc] init];
        center.delegate = delegate;

        // Ensure we have authorization (request if not determined).
        dispatch_semaphore_t authSema = dispatch_semaphore_create(0);
        __block BOOL authorized = NO;
        __block NSString *authError = nil;

        [center getNotificationSettingsWithCompletionHandler:^(UNNotificationSettings *settings) {
            if (settings.authorizationStatus == UNAuthorizationStatusAuthorized ||
                settings.authorizationStatus == UNAuthorizationStatusProvisional) {
                authorized = YES;
                dispatch_semaphore_signal(authSema);
            } else if (settings.authorizationStatus == UNAuthorizationStatusNotDetermined) {
                UNAuthorizationOptions opts =
                    UNAuthorizationOptionAlert | UNAuthorizationOptionSound;
                [center requestAuthorizationWithOptions:opts
                                      completionHandler:^(BOOL granted, NSError *error) {
                    authorized = granted;
                    if (error) authError = error.localizedDescription;
                    dispatch_semaphore_signal(authSema);
                }];
            } else {
                authError = @"notifications denied by user";
                dispatch_semaphore_signal(authSema);
            }
        }];
        dispatch_semaphore_wait(authSema, DISPATCH_TIME_FOREVER);

        if (!authorized) {
            respondAndExit(@"error",
                           authError ?: @"notification authorization not granted");
        }

        // Build the notification content.
        UNMutableNotificationContent *content = [[UNMutableNotificationContent alloc] init];
        content.title = title;
        content.body  = message;
        if (subtitle) content.subtitle = subtitle;

        // Sound handling.
        if (!sound ||
            [sound caseInsensitiveCompare:@"default"] == NSOrderedSame) {
            content.sound = [UNNotificationSound defaultSound];
        } else if ([sound caseInsensitiveCompare:@"none"] == NSOrderedSame ||
                   [sound caseInsensitiveCompare:@"off"] == NSOrderedSame ||
                   [sound caseInsensitiveCompare:@"silent"] == NSOrderedSame) {
            content.sound = nil;
        } else {
            content.sound = [UNNotificationSound soundNamed:sound];
        }

        // Create and submit the notification request.
        NSString *identifier = [[NSUUID UUID] UUIDString];
        UNNotificationRequest *request =
            [UNNotificationRequest requestWithIdentifier:identifier
                                                 content:content
                                                 trigger:nil]; // immediate

        dispatch_semaphore_t sendSema = dispatch_semaphore_create(0);
        __block NSString *sendError = nil;

        [center addNotificationRequest:request withCompletionHandler:^(NSError *error) {
            if (error) sendError = error.localizedDescription;
            dispatch_semaphore_signal(sendSema);
        }];
        dispatch_semaphore_wait(sendSema, DISPATCH_TIME_FOREVER);

        if (sendError) {
            respondAndExit(@"error", sendError);
        }

        // If we don't need to wait, report delivered and exit.
        if (!waitForClick) {
            respondAndExit(@"delivered", nil);
        }

        // Wait for the user to interact with the notification.
        __block BOOL responded = NO;
        delegate.onResponse = ^(NSString *status) {
            respond(status, nil);
            responded = YES;
            CFRunLoopStop(CFRunLoopGetMain());
        };

        // Run the event loop so the delegate receives callbacks.
        // Time out after 5 minutes to avoid hanging forever.
        CFRunLoopRunInMode(kCFRunLoopDefaultMode, 300.0, false);

        if (!responded) {
            respondAndExit(@"delivered", nil);
        }
    }
    return 0;
}
