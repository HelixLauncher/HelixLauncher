use std::{
    ffi::{c_int, c_uchar},
    io::{stdin, BufRead},
    ptr::{null, null_mut},
};

use local_encoding_ng::Encoder;

/*#[repr(C)]
struct JavaVMOption {
    optionString: *const c_char,
    extraInfo: *const c_void,
}

const JNI_VERSION_1_6: i32 = 0x00010006;

#[repr(C)]
struct JavaVMInitArgs {
    version: i32,

    nOptions: i32,
    options: *const JavaVMOption,
    ignoreUnrecognized: bool,
}

#[repr(C)]
struct JNIInvokeInterface_ {
    reserved0: *const c_void,
    reserved1: *const c_void,
    reserved2: *const c_void,

    destroy_java_vm: unsafe extern "system" fn(vm: *const JavaVM) -> i32,
    attach_current_thread: unsafe extern "system" fn(
        vm: *const JavaVM,
        penv: *mut *const JNIEnv,
        args: *const c_void,
    ) -> i32,
    detach_current_thread: unsafe extern "system" fn(vm: *const JavaVM),
    get_env: unsafe extern "system" fn(vm: *const JavaVM, *mut *const JNIEnv, version: i32) -> i32,
    attach_current_thread_as_daemon: unsafe extern "system" fn(
        vm: *const JavaVM,
        *mut *const JNIEnv,
        args: *const c_void,
    ) -> i32,
}

#[repr(C)]
struct JNINativeInterface_ {
    reserved0: *const c_void,
    reserved1: *const c_void,
    reserved2: *const c_void,

    reserved3: *const c_void,
    GetVersion: unsafe extern "system" fn(*const JNIEnv) -> i32,
}

type JavaVM = *const JNIInvokeInterface_;
type JNIEnv = *const JNINativeInterface_;*/

fn main() {
    let (jli_path, argv, is_jli) = {
        let mut buf = Vec::new();
        let mut jli_path = None;
        let mut argv = vec![String::from("java")];
        let mut stdin = stdin().lock();
        let mut is_jli = true;
        loop {
            stdin.read_until(0, &mut buf).unwrap();
            if buf.is_empty() {
                break;
            }
            match buf[0] {
                // TODO: should we pass the full jli library file path, the library directory path,
                //       or the java home?
                b'j' => jli_path = Some(String::from_utf8(buf[1..].to_vec()).unwrap()),
                b'a' => argv.push(String::from_utf8(buf[1..].to_vec()).unwrap()),
                b'o' => is_jli = false,
                _ => panic!("Incompatible Java wrapper"),
            }
            buf.clear();
        }
        (jli_path.expect("JLI path not passed"), argv, is_jli)
    };
    println!("{:?}", argv);
    if is_jli {
        let mut argv: Vec<_> = argv
            .into_iter()
            .map(|s| {
                let mut bytes = local_encoding_ng::Encoding::ANSI.to_bytes(&s).unwrap();
                bytes.reserve_exact(1);
                bytes.push(0);
                bytes.shrink_to_fit();
                bytes
            })
            .collect();
        let jli = unsafe { libloading::Library::new(jli_path) }.unwrap();
        let mut cargv: Vec<_> = argv.iter_mut().map(|v| v.as_mut_ptr()).collect();
        #[allow(clippy::type_complexity)]
        let jli_launch: libloading::Symbol<
            unsafe extern "system" fn(
                argc: c_int,
                argv: *mut *mut c_uchar,
                jargc: c_int,
                jargv: *mut *const c_uchar,
                appclassc: c_int,
                appclassv: *mut *const c_uchar,
                fullversion: *const c_uchar,
                dotversion: *const c_uchar,
                pname: *const c_uchar,
                lname: *const c_uchar,
                javaargs: bool,
                cpwildcard: bool,
                javaw: bool,
                ergo: i32,
            ) -> c_int,
        > = unsafe { jli.get(b"JLI_Launch") }.unwrap();

        unsafe {
            jli_launch(
                cargv.len() as c_int,
                cargv.as_mut_ptr(),
                0,
                null_mut(),
                0,
                null_mut(),
                null(),
                null(),
                b"java\0".as_ptr(),
                b"java\0".as_ptr(),
                false,
                false,
                false,
                0,
            );
        }
    } else {
        let mut args = jni::InitArgsBuilder::new();
        for arg in argv {
            args = args.option(arg);
        }
        let vm = jni::JavaVM::with_libjvm(args.build().unwrap(), || Ok(jli_path)).unwrap();
        let env = vm.attach_current_thread().unwrap();
        todo!();
        /*let jni_create_java_vm: libloading::Symbol<
            unsafe extern "system" fn(
                vm: *mut *const JavaVM,
                penv: *mut *const JNIEnv,
                args: *const JavaVMInitArgs,
            ),
        > = unsafe { jli.get(b"JNI_CreateJavaVM") }.unwrap();
        let mut vm: *const JavaVM = null();
        let mut env: *const JNIEnv = null();
        let options: Vec<_> = argv
            .iter()
            .skip(1)
            .map(|s| JavaVMOption {
                optionString: s.as_ptr() as _,
                extraInfo: null(),
            })
            .collect();
        let vm_args = JavaVMInitArgs {
            version: JNI_VERSION_1_6,
            nOptions: options.len() as i32,
            options: options.as_ptr(),
            ignoreUnrecognized: false,
        };
        unsafe { jni_create_java_vm(&mut vm, &mut env, &vm_args) };
        unsafe { ((**vm).destroy_java_vm)(vm)};*/
    }
}
