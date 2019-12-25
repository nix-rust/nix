//! Manipulate the contents of the shadow password file, `/etc/shadow`.
use std::ffi::CStr;
use std::ffi::CString;

/// Represents an entry in `/etc/shadow`.
// Documentation is based on the `shadow(5)` and `shadow(3)` man pages.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Shadow {
    /// User name
    pub name: String,
    /// Encrypted password
    ///
    /// Refer to crypt(3) for details on how this string is interpreted.
    /// If the password field contains some string that is not a valid
    /// result of crypt(3), for instance ! or *, the user will not be able to
    /// use a unix password to log in (but the user may log in the system by
    /// other means).
    ///
    /// This field may be empty, in which case no passwords are required to
    /// authenticate as the specified login name. However, some applications
    /// which read the /etc/shadow file may decide not to permit any access at
    /// all if the password field is empty.
    ///
    /// A password field which starts with a exclamation mark means that the
    /// password is locked.
    pub password: CString,
    /// Date of the last password change
    ///
    /// It is expressed as the number of days since Jan 1, 1970. The value 0
    /// has a special meaning, which is that the user should change their
    /// password the next time they will log in the system
    pub last_change: libc::c_long,
    /// Minimum password age
    ///
    /// The minimum password age is the number of days the user will have to
    /// wait before she will be allowed to change their password again.
    /// An empty field and value 0 mean that there are no minimum password age.
    pub min: libc::c_long,
    /// Maximum password age
    ///
    /// The maximum password age is the number of days after which the user
    /// will have to change their password.
    ///
    /// After this number of days is elapsed, the password may still be valid.
    /// The user should be asked to change their password the next time they
    /// will log in.
    ///
    /// If the maximum password age is lower than the minimum password age, the
    /// user cannot change their password.
    pub max: libc::c_long,
    /// Password warning period
    ///
    /// The number of days before a password is going to expire (see the
    /// maximum password age above) during which the user should be warned.
    ///
    /// An empty field and value 0 mean that there are no password warning
    /// period.
    pub warn: libc::c_long,
    /// Password inactivity period
    ///
    /// The number of days after a password has expired (see the maximum
    /// password age above) during which the password should still be accepted
    /// (and the user should update their password during the next login).
    ///
    /// After expiration of the password and this expiration period is elapsed,
    /// no login is possible using the current user's password. The user should
    /// contact their system administrator.
    pub inactive: libc::c_long,
    /// Account expiration date
    ///
    /// The date of expiration of the account, expressed as the number of days
    /// since Jan 1, 1970.
    ///
    /// Note that an account expiration differs from a password expiration. In
    /// case of an account expiration, the user shall not be allowed to login.
    /// In case of a password expiration, the user is not allowed to login using
    /// their password.
    ///
    /// The value 0 should not be used as it is interpreted as either an account
    /// with no expiration, or as an expiration on Jan 1, 1970.
    pub expire: libc::c_long,
}

impl From<&libc::spwd> for Shadow {
    fn from(spwd: &libc::spwd) -> Shadow {
        Shadow {
            name: unsafe { CStr::from_ptr(spwd.sp_namp).to_string_lossy().into_owned() },
            password: unsafe { CString::new(CStr::from_ptr(spwd.sp_pwdp).to_bytes()).unwrap() },
            last_change: spwd.sp_lstchg,
            min: spwd.sp_min,
            max: spwd.sp_max,
            warn: spwd.sp_warn,
            inactive: spwd.sp_inact,
            expire: spwd.sp_expire,
        }
    }
}

impl Shadow {
    /// Gets a [`Shadow`] entry for the given username, or returns [`None`].
    ///
    /// # Safety
    /// Care should be taken when this function is used in different threads
    /// without synchronization. This is because the underlying function used
    /// ([`getspnam()`][1]) is not thread safe.
    ///
    /// [1]: http://man7.org/linux/man-pages/man3/shadow.3.html
    pub fn from_name(user: &str) -> Option<Shadow> {
        let c_user = CString::new(user).unwrap();

        let spwd = unsafe { libc::getspnam(c_user.as_ptr()) };

        if spwd.is_null() {
            None
        } else {
            Some(unsafe { Shadow::from(&*spwd) })
        }
    }

    /// Returns iterator over all entries in `shadow` file
    pub fn iter_all() -> ShadowIterator {
        ShadowIterator::default()
    }
}

/// Iterator over `Shadow` entries
///
/// # Examples
/// ```
/// # use nix::shadow::ShadowIterator;
/// # use nix::shadow::Shadow;
/// let shadows = ShadowIterator::default().collect::<Vec<Shadow>>();
/// println!("There are {} shadow entries", shadows.len());
/// ```
///
/// # Safety
/// Care should be taken when this iterator is used in different threads
/// without synchronization. This is because the underlying functions used
/// ([`getspent()`][1], and [`endspent()`][1]) are not thread safe.
///
/// [1]: http://man7.org/linux/man-pages/man3/shadow.3.html
#[derive(Debug, Default)]
pub struct ShadowIterator {
    started: bool,
    done: bool,
}

impl Iterator for ShadowIterator {
    type Item = Shadow;

    fn next(&mut self) -> Option<Shadow> {
        self.started = true;
        if !self.done {
            let spwd = unsafe { libc::getspent() };
            if spwd.is_null() {
                unsafe { libc::endspent() };
                self.done = true;
                None
            } else {
                Some(unsafe { Shadow::from(&*spwd) })
            }
        } else {
            None
        }
    }
}

impl Drop for ShadowIterator {
    fn drop(&mut self) {
        if self.started && !self.done {
            unsafe { libc::endspent() };
        }
    }
}
