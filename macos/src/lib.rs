// Copyright 2016 Avraham Weinstock
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use objc2::rc::Id;
use objc2::runtime::{AnyClass, AnyObject, ProtocolObject};
use objc2::{msg_send_id, ClassType};
use objc2_app_kit::{NSPasteboard, NSPasteboardItem};
use objc2_foundation::{NSArray, NSData, NSString};
use std::error::Error;
use std::panic::{RefUnwindSafe, UnwindSafe};

pub struct Clipboard {
    pasteboard: Id<NSPasteboard>,
}

unsafe impl Send for Clipboard {}
unsafe impl Sync for Clipboard {}
impl UnwindSafe for Clipboard {}
impl RefUnwindSafe for Clipboard {}

impl Clipboard {
    pub fn new() -> Result<Clipboard, Box<dyn Error>> {
        // Use `msg_send_id!` instead of `NSPasteboard::generalPasteboard()`
        // in the off case that it will return NULL (even though it's
        // documented not to).
        let pasteboard: Option<Id<NSPasteboard>> =
            unsafe { msg_send_id![NSPasteboard::class(), generalPasteboard] };
        let pasteboard =
            pasteboard.ok_or("NSPasteboard#generalPasteboard returned null")?;
        Ok(Self { pasteboard })
    }

    pub fn read(&self) -> Result<String, Box<dyn Error>> {
        // The NSPasteboard API is a bit weird, it requires you to pass
        // classes as objects, which `objc2_foundation::NSArray` was not really
        // made for - so we convert the class to an `AnyObject` type instead.
        //
        // TODO: Use the NSPasteboard helper APIs (`stringForType`).
        let string_class = {
            let cls: *const AnyClass = NSString::class();
            let cls = cls as *mut AnyObject;
            unsafe { Id::retain(cls).unwrap() }
        };
        let classes = NSArray::from_vec(vec![string_class]);
        let string_array = unsafe {
            self.pasteboard
                .readObjectsForClasses_options(&classes, None)
        }
        .ok_or("pasteboard#readObjectsForClasses:options: returned null")?;

        let obj: *const AnyObject = string_array.first().ok_or(
            "pasteboard#readObjectsForClasses:options: returned empty",
        )?;
        // And this part is weird as well, since we now have to convert the object
        // into an NSString, which we know it to be since that's what we told
        // `readObjectsForClasses:options:`.
        let obj: *mut NSString = obj as _;
        Ok(unsafe { Id::retain(obj) }.unwrap().to_string())
    }

    pub fn read_data(&self) -> Result<(String, Vec<u8>), Box<dyn Error>> {
        // The NSPasteboard API is a bit weird, it requires you to pass
        // classes as objects, which `objc2_foundation::NSArray` was not really
        // made for - so we convert the class to an `AnyObject` type instead.
        //
        // TODO: Use the NSPasteboard helper APIs (`stringForType`).
        let string_class = {
            let cls: *const AnyClass = NSPasteboardItem::class();
            let cls = cls as *mut AnyObject;
            unsafe { Id::retain(cls).unwrap() }
        };
        let classes = NSArray::from_vec(vec![string_class]);
        let string_array = unsafe {
            self.pasteboard
                .readObjectsForClasses_options(&classes, None)
        }
        .ok_or("pasteboard#readObjectsForClasses:options: returned null")?;

        let obj: *const AnyObject = string_array.first().ok_or(
            "pasteboard#readObjectsForClasses:options: returned empty",
        )?;
        // And this part is weird as well, since we now have to convert the object
        // into an NSString, which we know it to be since that's what we told
        // `readObjectsForClasses:options:`.
        let obj: *mut NSPasteboardItem = obj as _;
        let ss = unsafe { Id::retain(obj) }.unwrap();
        let ns1 = unsafe {
            ss.dataForType(&NSString::from_str("public.utf8-plain-text"))
        }
        .unwrap();
        let ns2 = unsafe {
            ss.dataForType(&NSString::from_str(
                "com.kakao.kakaoTalk.emoji.attachment",
            ))
        }
        .unwrap();
        // let ss = unsafe { ss.types() };
        // let mut v = Vec::new();
        // for i in 0..ss.count() {
        //     let s = ss.get(i).unwrap();
        //     v.push(s.to_string());
        // }
        Ok((String::from_utf8_lossy(&ns1.bytes()).to_string(), ns2.bytes().to_vec()))
    }

    pub fn read_buffer(&self) -> Result<Vec<String>, Box<dyn Error>> {
        // The NSPasteboard API is a bit weird, it requires you to pass
        // classes as objects, which `objc2_foundation::NSArray` was not really
        // made for - so we convert the class to an `AnyObject` type instead.
        //
        // TODO: Use the NSPasteboard helper APIs (`stringForType`).
        let string_class = {
            let cls: *const AnyClass = NSPasteboardItem::class();
            let cls = cls as *mut AnyObject;
            unsafe { Id::retain(cls).unwrap() }
        };
        let classes = NSArray::from_vec(vec![string_class]);
        let string_array = unsafe {
            self.pasteboard
                .readObjectsForClasses_options(&classes, None)
        }
        .ok_or("pasteboard#readObjectsForClasses:options: returned null")?;

        let obj: *const AnyObject = string_array.first().ok_or(
            "pasteboard#readObjectsForClasses:options: returned empty",
        )?;
        // And this part is weird as well, since we now have to convert the object
        // into an NSString, which we know it to be since that's what we told
        // `readObjectsForClasses:options:`.
        let obj: *mut NSPasteboardItem = obj as _;
        let ss = unsafe { Id::retain(obj) }.unwrap();
        let ss = unsafe { ss.types() };
        let mut v = Vec::new();
        for i in 0..ss.count() {
            let s = ss.get(i).unwrap();
            v.push(s.to_string());
        }
        Ok(v)
    }

    pub fn write(&mut self, data: String) -> Result<(), Box<dyn Error>> {
        let string_array = NSArray::from_vec(vec![ProtocolObject::from_id(
            NSString::from_str(&data),
        )]);
        unsafe { self.pasteboard.clearContents() };
        let success = unsafe { self.pasteboard.writeObjects(&string_array) };
        if success {
            Ok(())
        } else {
            Err("NSPasteboard#writeObjects: returned false".into())
        }
    }

    pub fn write_data(
        &mut self,
        s: &str,
        data: Vec<u8>,
    ) -> Result<(), Box<dyn Error>> {
        let p1 = s.as_ptr() as *mut std::ffi::c_void;
        let ns_data1 = unsafe {
            NSData::initWithBytes_length(
                NSData::alloc(),
                p1,
                s.as_bytes().len(),
            )
        };
        let p2 = data.as_ptr() as *mut std::ffi::c_void;
        let ns_data2 = unsafe {
            NSData::initWithBytes_length(NSData::alloc(), p2, data.len())
        };
        let item = unsafe { NSPasteboardItem::init(NSPasteboardItem::alloc()) };
        unsafe {
            item.setData_forType(
                &ns_data1,
                &NSString::from_str("public.utf8-plain-text"),
            )
        };
        unsafe {
            item.setData_forType(
                &ns_data2,
                &NSString::from_str("com.kakao.kakaoTalk.emoji.attachment"),
            )
        };
        let string_array =
            NSArray::from_vec(vec![ProtocolObject::from_id(item)]);
        unsafe { self.pasteboard.clearContents() };
        let success = unsafe { self.pasteboard.writeObjects(&string_array) };
        if success {
            Ok(())
        } else {
            Err("NSPasteboard#writeObjects: returned false".into())
        }
    }
}
