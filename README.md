# tid-rs

TouchId integration for Rust

## Usage

```rust
async fn touch_id() {
    let mut ctx = LAContext::new();
    if ctx.can_evaluate_policy(LAPolicy::DeviceOwnerAuthenticationWithBiometrics) {
        ctx.set_localized_cancel_title("Use Another Method");
        ctx.evaluate_policy(
            LAPolicy::DeviceOwnerAuthenticationWithBiometrics,
            "Use TouchId to Unlock Rust",
        ).await;
    }
}
```