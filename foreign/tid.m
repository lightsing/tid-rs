#import "tid.h"
#import <LocalAuthentication/LocalAuthentication.h>

void* create_la_context() {
    return (__bridge void*)[[LAContext alloc] init];
}

void drop_la_context(void* ctx) {
    [(__bridge LAContext*)ctx release];
}

void set_localized_cancel_title(void* ctx, char* reason) {
    [(__bridge LAContext*)ctx setLocalizedCancelTitle:[[NSString alloc] initWithCString:reason]];
}

int32_t can_evaluate_policy(void *ctx, int32_t policy) {
    BOOL result = [(__bridge LAContext*)ctx canEvaluatePolicy:(LAPolicy)policy error:nil];
    return result;
}

void evaluate_policy(void* ctx, int32_t policy, char* reason, void* future, void* callback) {
    NSString* reasonString = [[NSString alloc] initWithCString:reason encoding:NSUTF8StringEncoding];
    [
        (__bridge LAContext*)ctx
        evaluatePolicy:(LAPolicy)policy
        localizedReason:reasonString
        reply:^(BOOL success, NSError *error) {
            if (!success && error != nil) {
                ((void (*)(void*, BOOL, int32_t))callback)(future, success, (int32_t)error.code);
            } else if (success && error == nil) {
                ((void (*)(void*, BOOL, int32_t))callback)(future, success, 0);
            }
        }
    ];
}
