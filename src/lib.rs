mod ffi {
    #![allow(
        dead_code,
        non_upper_case_globals,
        non_camel_case_types,
        non_snake_case
    )]
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

#[derive(Debug, thiserror::Error)]
pub enum XcbError {
    #[error("{0}")]
    GenericError(String),
    #[error("{0}")]
    ReplyError(String),
    #[error("{0}")]
    ConnectionError(xcb::ConnError),
}
impl XcbError {
    pub fn from_generic_reply(context: &ErrorContext, e: xcb::ReplyError) -> Self {
        Self::GenericError(context.reply_message(e))
    }
    pub fn from_generic<T>(context: &ErrorContext, e: xcb::Error<T>) -> Self {
        Self::GenericError(context.generic_message(e))
    }
}
impl From<xcb::ConnError> for XcbError {
    fn from(e: xcb::ConnError) -> Self {
        Self::ConnectionError(e)
    }
}

pub struct ErrorContext {
    context: *mut ffi::xcb_errors_context_t,
}
impl ErrorContext {
    pub fn reply_message(&self, e: xcb::ReplyError) -> String {
        match e {
            xcb::ReplyError::NullResponse => format!("Null Response"),
            xcb::ReplyError::GenericError(e) => unsafe {
                let error_str = {
                    let str = ffi::xcb_errors_get_name_for_error(
                        self.context,
                        e.error_code(),
                        std::ptr::null_mut(),
                    );

                    if str.is_null() {
                        return format!("Unknown Error: {}", e);
                    }
                    std::ffi::CStr::from_ptr(str).to_str().unwrap()
                };
                let major_str = {
                    let str = ffi::xcb_errors_get_name_for_major_code(self.context, e.major_code());

                    if str.is_null() {
                        "()"
                    } else {
                        std::ffi::CStr::from_ptr(str).to_str().unwrap()
                    }
                };
                let minor_str = {
                    let str = ffi::xcb_errors_get_name_for_minor_code(
                        self.context,
                        e.major_code(),
                        e.minor_code(),
                    );

                    if str.is_null() {
                        "()"
                    } else {
                        std::ffi::CStr::from_ptr(str).to_str().unwrap()
                    }
                };

                format!(
                    "Error: {}, Major:{}, Minor:{}",
                    error_str, major_str, minor_str
                )
            },
        }
    }

    pub fn generic_message<T>(&self, e: xcb::Error<T>) -> String {
        unsafe {
            let str = ffi::xcb_errors_get_name_for_error(
                self.context,
                e.error_code(),
                std::ptr::null_mut(),
            );

            if str.is_null() {
                return format!("Unknown Error: {}", e);
            }

            format!(
                "{} - {:?}",
                std::ffi::CStr::from_ptr(str).to_str().unwrap().to_string(),
                e.to_string()
            )
        }
    }

    pub fn new(connection: *mut xcb::ffi::xcb_connection_t) -> Result<Self, XcbError> {
        let context = unsafe {
            let mut context = std::ptr::null_mut();
            let _ = ffi::xcb_errors_context_new(connection as *mut _, &mut context as *mut _);
            context
        };

        Ok(Self { context })
    }
}
impl Drop for ErrorContext {
    fn drop(&mut self) {}
}
