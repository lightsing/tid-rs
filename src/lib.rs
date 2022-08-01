use std::ffi::{c_void, CString};
use std::os::raw::c_char;
use num::FromPrimitive;
use tokio::sync::oneshot::Sender;

extern "C" {
    fn create_la_context() -> *mut c_void;
    fn drop_la_context(ctx: *mut c_void);
    fn set_localized_cancel_title(ctx: *mut c_void, reason: *const c_char);
    fn can_evaluate_policy(ctx: *mut c_void, policy: i32) -> i32;
    fn evaluate_policy(
        ctx: *mut c_void,
        policy: i32,
        reason: *const c_char,
        user_data: *mut c_void,
        callback: *mut c_void,
    );
}

#[repr(i32)]
#[derive(Copy, Clone, Debug)]
pub enum LAPolicy {
    DeviceOwnerAuthenticationWithBiometrics = 1,
    DeviceOwnerAuthentication = 2,
    DeviceOwnerAuthenticationWithWatch = 3,
    DeviceOwnerAuthenticationWithBiometricsOrWatch = 4,
    DeviceOwnerAuthenticationWithWristDetection = 5,
}

#[repr(i32)]
#[derive(Copy, Clone, Debug, num_derive::FromPrimitive, thiserror::Error)]
pub enum LAError {
    #[error("The app canceled authentication.")]
    LAErrorAppCancel = -9,
    #[error("The system canceled authentication.")]
    LAErrorSystemCancel = -4,
    #[error("The user tapped the cancel button in the authentication dialog.")]
    LAErrorUserCancel = -2,
    #[error("The device supports biometry only using a removable accessory, but the paired accessory isn’t connected.")]
    LAErrorBiometryDisconnected = -13,
    #[error("Biometry is locked because there were too many failed attempts.")]
    LAErrorBiometryLockout = -8,
    #[error("Biometry is not available on the device.")]
    LAErrorBiometryNotAvailable = -6,
    #[error("The user has no enrolled biometric identities.")]
    LAErrorBiometryNotEnrolled = -7,
    #[error("The device supports biometry only using a removable accessory, but no accessory is paired.")]
    LAErrorBiometryNotPaired = -12,
    #[error("The user failed to provide valid credentials.")]
    LAErrorAuthenticationFailed = -1,
    #[error("The context was previously invalidated.")]
    LAErrorInvalidContext = -10,
    #[error("kLAErrorInvalidDimensions")]
    LAErrorInvalidDimensions = -14,
    #[error("Displaying the required authentication user interface is forbidden.")]
    LAErrorNotInteractive = -1004,
    #[error("A passcode isn’t set on the device.")]
    LAErrorPasscodeNotSet = -5,
    #[error("The user tapped the fallback button in the authentication dialog, but no fallback is available for the authentication policy.")]
    LAErrorUserFallback = -3,
    #[error("An attempt to authenticate with Apple Watch failed.")]
    LAErrorWatchNotAvailable = -11,
}

pub struct LAContext {
    inner: *mut c_void,
}

impl LAContext {
    pub fn new() -> Self {
        let ctx = unsafe { create_la_context() };
        Self { inner: ctx }
    }

    pub fn set_localized_cancel_title(&mut self, title: &str) {
        let title = CString::new(title).unwrap();
        unsafe {
            set_localized_cancel_title(self.inner, title.as_ptr());
        }
    }

    pub fn can_evaluate_policy(&self, policy: LAPolicy) -> bool {
        unsafe {
            can_evaluate_policy(self.inner, policy as i32) == 1
        }
    }

    pub async fn evaluate_policy(
        &self,
        policy: LAPolicy,
        localized_reason: &str,
    ) -> Result<(), LAError> {
        let reason = CString::new(localized_reason).unwrap();
        let (tx, rx) = tokio::sync::oneshot::channel();
        let tx = Box::into_raw(Box::new(tx));
        unsafe {
            evaluate_policy(
                self.inner,
                policy as i32,
                reason.as_ptr(),
                tx as *mut c_void,
                evaluate_callback as *mut c_void,
            );
        }
        rx.await.unwrap()
    }
}

impl Drop for LAContext {
    fn drop(&mut self) {
        unsafe {
            drop_la_context(self.inner);
        }
    }
}

unsafe extern "C" fn evaluate_callback(tx: *mut c_void, success: i32, code: i32) {
    let tx = Box::from_raw(tx as *mut Sender<Result<(), LAError>>);
    if success == 1 {
        tx.send(Ok(())).unwrap();
    } else {
        let error: LAError = FromPrimitive::from_i32(code).unwrap();
        tx.send(Err(error)).unwrap();
    }
}