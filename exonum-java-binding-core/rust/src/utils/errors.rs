use jni::objects::JObject;
use jni::JNIEnv;

use std::cell::Cell;

use utils::{get_class_name, get_exception_message};
use {JniErrorKind, JniResult};

const CLASS_JL_ERROR: &str = "java/lang/Error";

/// Unwraps the result, returning its content.
///
/// Panics:
/// - Panics if there is some JNI error.
/// - If there is a pending Java exception of any type,
///   handles it and panics with a message from the exception.
pub fn panic_on_exception<T>(env: &JNIEnv, result: JniResult<T>) -> T {
    result.unwrap_or_else(|jni_error| match jni_error.0 {
        JniErrorKind::JavaException => {
            let exception = get_and_clear_java_exception(env);
            panic!(describe_java_exception(env, exception));
        }
        _ => unwrap_jni(Err(jni_error)),
    })
}

/// Handles and describes non-fatal Java exceptions.
///
/// Java exceptions are converted into `Error`s with their descriptions, Java errors and JNI errors
/// are treated as unrecoverable and result in a panic.
///
/// Panics:
/// - Panics if there is some JNI error.
/// - If there is a pending Java exception that is a subclass of `java.lang.Error`.
pub fn check_error_on_exception<T>(env: &JNIEnv, result: JniResult<T>) -> Result<T, String> {
    result.map_err(|jni_error| match jni_error.0 {
        JniErrorKind::JavaException => {
            let exception = get_and_clear_java_exception(env);
            let message = describe_java_exception(env, exception);
            if unwrap_jni_verbose(env, env.is_instance_of(exception, CLASS_JL_ERROR)) {
                panic!(message);
            }
            message
        }
        _ => unwrap_jni(Err(jni_error)),
    })
}

/// Unwraps `jni::Result`
///
/// Panics if there is some JNI error.
pub fn unwrap_jni<T>(res: JniResult<T>) -> T {
    res.unwrap_or_else(|err| panic!("JNI error: {:?}", err))
}

/// Unwraps `jni::Result` with verbose error message if Java exception occurred.
/// To get an additional info about the exception, it calls JNI API, which can lead
/// to another exception. In that case it gives up to get verbose error message to prevent
/// an infinite recursion and stack overflow.
///
/// Panics if there is some JNI error.
pub fn unwrap_jni_verbose<T>(env: &JNIEnv, res: JniResult<T>) -> T {
    thread_local! {
        static IN_RECURSION: Cell<bool> = Cell::new(false);
    }
    IN_RECURSION.with(|in_recursion| {
        res.unwrap_or_else(|jni_error| {
            // If we get another JNI error whilst handling one — stop processing both and panic.
            if in_recursion.get() {
                // Reset the flag to allow future use of this method.
                in_recursion.set(false);
                panic!("Recursive JNI error: {:?}", jni_error);
            } else {
                match jni_error.0 {
                    JniErrorKind::JavaException => {
                        in_recursion.set(true);
                        let exception = get_and_clear_java_exception(env);
                        let message = describe_java_exception(env, exception);
                        in_recursion.set(false);
                        panic!(message);
                    }
                    _ => unwrap_jni(Err(jni_error)),
                }
            }
        })
    })
}

/// Returns (and clears) any exception that is currently being thrown.
pub fn get_and_clear_java_exception<'e>(env: &'e JNIEnv) -> JObject<'e> {
    let exception: JObject = unwrap_jni(env.exception_occurred()).into();
    // A null exception from #exception_occurred indicates that there is no pending exception.
    // It is possible if current thread is reattached to JVM.
    assert!(!exception.is_null(), "No exception thrown.");
    unwrap_jni_verbose(env, env.exception_clear());
    exception
}

/// Describes Java exception in a form of "Java ecception: EXCEPTION_NAME: EXCEPTION_DETAILS"
pub fn describe_java_exception(env: &JNIEnv, exception: JObject) -> String {
    assert!(!exception.is_null(), "No exception thrown.");
    let format = || {
        Ok(format!(
            "Java exception: {}; {:?}",
            get_class_name(env, exception)?,
            get_exception_message(env, exception)?,
        ))
    };
    unwrap_jni_verbose(env, format())
}
