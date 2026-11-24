//! What a way to reach a name is.

use crate::code::codes;

codes! {
    /// What a way to reach a name is. These are the 24 that QuickBooks starts
    /// with, and a company can add others.
    pub enum ChannelKind {
        /// The main phone number.
        MainPhone = 1,
        /// The phone number at work.
        WorkPhone = 2,
        /// The phone number at home.
        HomePhone = 3,
        /// The number of a mobile phone.
        Mobile = 4,
        /// The number of a fax.
        Fax = 5,
        /// Another phone number.
        AltPhone = 6,
        /// The number of another mobile phone.
        AltMobile = 7,
        /// The number of another fax.
        AltFax = 8,
        /// The main e-mail address.
        MainEmail = 9,
        /// The address that e-mails are copied to.
        CcEmail = 10,
        /// Another e-mail address.
        AltEmail1 = 11,
        /// Another e-mail address.
        AltEmail2 = 12,
        /// The address of a website.
        Website = 13,
        /// A page on LinkedIn.
        LinkedIn = 14,
        /// A page on Facebook.
        Facebook = 15,
        /// A name on Twitter.
        Twitter = 16,
        /// The address of a page.
        Url1 = 17,
        /// The address of a page.
        Url2 = 18,
        /// The address of a page.
        Url3 = 19,
        /// The address of a page.
        Url4 = 20,
        /// A name on Skype.
        Skype = 21,
        /// Anything else.
        Other1 = 22,
        /// Anything else.
        Other2 = 23,
        /// Anything else.
        Other3 = 24,
    }
}
