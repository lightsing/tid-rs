use tid::{LAContext, LAPolicy};

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let mut ctx = LAContext::new();

    ctx.set_localized_cancel_title("Use Another Method");
    if ctx.can_evaluate_policy(LAPolicy::DeviceOwnerAuthenticationWithBiometrics) {
        println!("device supports biometrics authentication");
        let auth_result = ctx.evaluate_policy(
            LAPolicy::DeviceOwnerAuthenticationWithBiometrics,
            "Use TouchId to Unlock Rust",
        ).await;
        println!("Authentication result: {:?}", auth_result);
    }
}