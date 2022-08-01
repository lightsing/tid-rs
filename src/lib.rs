//! TouchId integration for Rust
//!
//! This crate provides touch-id integration.
//!
//! # Get Started
//!
//! Add following code to your `Cargo.toml`:
//! ```toml
//! tid-rs = "0.1"
//! ```
//! ## Example
//!
//! ```rust
//! use tid::{LAContext, LAPolicy};
//!
//! #[tokio::main(flavor = "current_thread")]
//! async fn main() {
//!     let mut ctx = LAContext::new();
//!
//!     ctx.set_localized_cancel_title("Use Another Method");
//!     if ctx.can_evaluate_policy(LAPolicy::DeviceOwnerAuthenticationWithBiometrics) {
//!         println!("device supports biometrics authentication");
//!         let auth_result = ctx.evaluate_policy(
//!             LAPolicy::DeviceOwnerAuthenticationWithBiometrics,
//!             "Use TouchId to Unlock Rust",
//!         ).await;
//!         println!("Authentication result: {:?}", auth_result);
//!     }
//! }
//! ```
#![deny(missing_docs)]

use num::FromPrimitive;
use parking_lot::Mutex;
use std::cell::Cell;
use std::ffi::{c_void, CString};
use std::future::Future;
use std::os::raw::c_char;
use std::pin::Pin;
use std::rc::Rc;
use std::task::{Context, Poll, Waker};

extern "C" {
    fn create_la_context() -> *mut c_void;
    fn drop_la_context(ctx: *mut c_void);
    fn set_localized_cancel_title(ctx: *mut c_void, reason: *const c_char);
    fn can_evaluate_policy(ctx: *mut c_void, policy: i32) -> i32;
    fn evaluate_policy(
        ctx: *mut c_void,
        policy: i32,
        reason: *const c_char,
        user_data: *const c_void,
        callback: *mut c_void,
    );
}

/// Binding to `LAPolicy` of `LocalAuthentication`
///
/// The set of available local authentication policies.
#[repr(i32)]
#[derive(Copy, Clone, Debug)]
pub enum LAPolicy {
    /// User authentication with biometry.
    DeviceOwnerAuthenticationWithBiometrics = 1,
    /// User authentication with Apple Watch.
    DeviceOwnerAuthentication = 2,
    /// User authentication with either biometry or Apple Watch.
    DeviceOwnerAuthenticationWithWatch = 3,
    /// User authentication with biometry, Apple Watch, or the device passcode.
    DeviceOwnerAuthenticationWithBiometricsOrWatch = 4,
    /// User authentication with wrist detection on watchOS.
    DeviceOwnerAuthenticationWithWristDetection = 5,
}

/// Binding to `LAError` of `LocalAuthentication`
///
/// Error codes that the framework returns when policy evaluation fails.
#[repr(i32)]
#[derive(Copy, Clone, Debug, num_derive::FromPrimitive, thiserror::Error)]
pub enum LAError {
    /// The app canceled authentication.
    #[error("The app canceled authentication.")]
    LAErrorAppCancel = -9,
    /// The system canceled authentication.
    #[error("The system canceled authentication.")]
    LAErrorSystemCancel = -4,
    /// The user tapped the cancel button in the authentication dialog.
    #[error("The user tapped the cancel button in the authentication dialog.")]
    LAErrorUserCancel = -2,
    /// The device supports biometry only using a removable accessory, but the paired accessory isn’t connected.
    #[error("The device supports biometry only using a removable accessory, but the paired accessory isn’t connected.")]
    LAErrorBiometryDisconnected = -13,
    /// Biometry is locked because there were too many failed attempts.
    #[error("Biometry is locked because there were too many failed attempts.")]
    LAErrorBiometryLockout = -8,
    /// Biometry is not available on the device.
    #[error("Biometry is not available on the device.")]
    LAErrorBiometryNotAvailable = -6,
    /// The user has no enrolled biometric identities.
    #[error("The user has no enrolled biometric identities.")]
    LAErrorBiometryNotEnrolled = -7,
    /// The device supports biometry only using a removable accessory, but no accessory is paired.
    #[error("The device supports biometry only using a removable accessory, but no accessory is paired.")]
    LAErrorBiometryNotPaired = -12,
    /// The user failed to provide valid credentials.
    #[error("The user failed to provide valid credentials.")]
    LAErrorAuthenticationFailed = -1,
    /// The context was previously invalidated.
    #[error("The context was previously invalidated.")]
    LAErrorInvalidContext = -10,
    /// undocumented
    #[error("kLAErrorInvalidDimensions")]
    LAErrorInvalidDimensions = -14,
    /// Displaying the required authentication user interface is forbidden.
    #[error("Displaying the required authentication user interface is forbidden.")]
    LAErrorNotInteractive = -1004,
    /// A passcode isn’t set on the device.
    #[error("A passcode isn’t set on the device.")]
    LAErrorPasscodeNotSet = -5,
    /// The user tapped the fallback button in the authentication dialog, but no fallback is available for the authentication policy.
    #[error("The user tapped the fallback button in the authentication dialog, but no fallback is available for the authentication policy.")]
    LAErrorUserFallback = -3,
    /// An attempt to authenticate with Apple Watch failed.
    #[error("An attempt to authenticate with Apple Watch failed.")]
    LAErrorWatchNotAvailable = -11,
}

/// Binding to `LAContext` of `LocalAuthentication`
///
/// A mechanism for evaluating authentication policies and access controls.
///
/// ## Overview
/// You use an authentication context to evaluate the user’s identity,
/// either with biometrics like Touch ID or Face ID, or by supplying the device passcode.
/// The context handles user interaction, and also interfaces to the Secure Enclave,
/// the underlying hardware element that manages biometric data.
/// You create and configure the context, and ask it to carry out the authentication.
/// You then receive an asynchronous callback,
/// which provides an indication of authentication success or failure,
/// and an error instance that explains the reason for a failure, if any.
///
/// ### Important
/// Include the [NSFaceIDUsageDescription](https://developer.apple.com/library/archive/documentation/General/Reference/InfoPlistKeyReference/Articles/CocoaKeys.html#//apple_ref/doc/uid/TP40009251-SW75) key in your app’s Info.plist file
/// if your app allows biometric authentication. Otherwise, authorization requests may fail.
///
pub struct LAContext {
    inner: *mut c_void,
}

impl LAContext {
    /// create a new `LAContext`
    pub fn new() -> Self {
        let ctx = unsafe { create_la_context() };
        Self { inner: ctx }
    }

    /// set `localizedCancelTitle` property.
    ///
    /// The localized title for the cancel button in the dialog presented to the user during authentication.
    ///
    /// ### Discussion
    /// The system presents a cancel button during biometric authentication
    /// to let the user abort the authentication attempt.
    /// The button appears every time the system asks the user
    /// to present a finger registered with Touch ID. For Face ID, the button only appears
    /// if authentication fails and the user is prompted to try again.
    /// Either way, the user can stop trying to authenticate by tapping the button.
    ///
    /// Use the localizedCancelTitle property to choose a title for the cancel button.
    /// If you set the property to nil—as it is by default—or assign an empty string,
    /// the system uses an appropriate default title, like “Cancel”.
    /// Otherwise, provide a localized string that’s short and clear.
    pub fn set_localized_cancel_title(&mut self, title: &str) {
        let title = CString::new(title).unwrap();
        unsafe {
            set_localized_cancel_title(self.inner, title.as_ptr());
        }
    }

    /// Assesses whether authentication can proceed for a given policy.
    pub fn can_evaluate_policy(&self, policy: LAPolicy) -> bool {
        unsafe { can_evaluate_policy(self.inner, policy as i32) == 1 }
    }

    /// Evaluates the specified policy.
    pub async fn evaluate_policy(
        &self,
        policy: LAPolicy,
        localized_reason: &str,
    ) -> Result<(), LAError> {
        let reason = CString::new(localized_reason).unwrap();
        // let (tx, rx) = tokio::sync::oneshot::channel();
        // let tx = Box::into_raw(Box::new(tx));
        let fut = EvaluateFuture::new();
        unsafe {
            evaluate_policy(
                self.inner,
                policy as i32,
                reason.as_ptr(),
                fut.inner.clone().into_raw() as *mut c_void,
                evaluate_callback as *mut c_void,
            );
        }
        fut.await
    }
}

impl Drop for LAContext {
    fn drop(&mut self) {
        unsafe {
            drop_la_context(self.inner);
        }
    }
}

struct EvaluateFuture {
    inner: Rc<EvaluateFutureInner>,
}

#[derive(Default)]
struct EvaluateFutureInner {
    result: Mutex<Cell<Option<Result<(), LAError>>>>,
    waker: Cell<Option<Waker>>,
}

impl EvaluateFuture {
    fn new() -> EvaluateFuture {
        EvaluateFuture {
            inner: Rc::new(EvaluateFutureInner::default()),
        }
    }
}

impl EvaluateFutureInner {
    fn into_raw(self: Rc<Self>) -> *const EvaluateFutureInner {
        Rc::into_raw(self)
    }
}

impl Future for EvaluateFuture {
    type Output = Result<(), LAError>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let guard = self.inner.result.lock();
        self.inner.waker.set(Some(cx.waker().clone()));
        if let Some(result) = guard.take() {
            Poll::Ready(result)
        } else {
            Poll::Pending
        }
    }
}

unsafe extern "C" fn evaluate_callback(tx: *const c_void, success: i32, code: i32) {
    let fut = Rc::from_raw(tx as *const EvaluateFutureInner);
    let guard = fut.result.lock();
    if success == 1 {
        guard.set(Some(Ok(())));
    } else {
        let error: LAError = FromPrimitive::from_i32(code).unwrap();
        guard.set(Some(Err(error)));
    }
    loop {
        if let Some(waker) = fut.waker.take() {
            waker.wake();
            break;
        }
        // the callback usually needs some time to be call (user need time to respond),
        // during that period the waker should already set.
    }
}
