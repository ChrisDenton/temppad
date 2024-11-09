// FIXME: the initial version was created in less than an hour so the code isn't exactly exemplary.
// At the very least the unsafe bits should be extracted and made safe (use an existing library?).

#![windows_subsystem = "windows"]
#![no_std]
#![no_main]

use core::{
    ffi::c_void,
    mem::{self, size_of},
    ptr::{addr_of, null_mut as null},
    sync::atomic::{AtomicPtr, Ordering},
};
use windows_sys::Win32::{
    Foundation::{HWND, LPARAM, LRESULT, TRUE, WPARAM},
    Graphics::Gdi::{
        CreateFontW, GetDC, GetTextExtentPoint32W, InflateRect, ReleaseDC, UpdateWindow,
        ANSI_CHARSET, CLIP_DEFAULT_PRECIS, COLOR_WINDOWFRAME, DEFAULT_PITCH, DEFAULT_QUALITY,
        FF_DONTCARE, FW_DONTCARE, OUT_TT_PRECIS,
    },
    System::LibraryLoader::{
        GetModuleHandleW, LoadLibraryExW, LOAD_LIBRARY_AS_DATAFILE, LOAD_LIBRARY_AS_IMAGE_RESOURCE,
        LOAD_LIBRARY_SEARCH_SYSTEM32,
    },
    UI::{
        Controls::{EM_SETRECT, EM_SETSEL, EM_SETTABSTOPS},
        Input::KeyboardAndMouse::SetFocus,
        WindowsAndMessaging::{
            CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW, GetClientRect,
            GetMessageW, GetWindowLongPtrW, LoadCursorW, LoadImageW, MoveWindow, PostQuitMessage,
            RegisterClassExW, SendMessageW, ShowWindow, TranslateMessage, CW_USEDEFAULT,
            ES_AUTOHSCROLL, ES_LEFT, ES_MULTILINE, ES_NOHIDESEL, GWLP_HINSTANCE, IDC_ARROW,
            IMAGE_ICON, LR_DEFAULTCOLOR, SW_SHOWDEFAULT, WM_CHAR, WM_CLOSE, WM_CREATE, WM_DESTROY,
            WM_SETFOCUS, WM_SETFONT, WM_SIZE, WNDCLASSEXW, WS_CHILD, WS_EX_CLIENTEDGE, WS_HSCROLL,
            WS_OVERLAPPEDWINDOW, WS_VISIBLE, WS_VSCROLL,
        },
    },
};

const CONTROL_A: usize = 1;
const ID_EDITCHILD: *mut c_void = 100 as *mut c_void;

static EDIT_CONTROL: AtomicPtr<c_void> = AtomicPtr::new(null());

#[no_mangle]
extern "C" fn main() -> i32 {
    let instance = unsafe { GetModuleHandleW(core::ptr::null()) };
    let icon_lib = unsafe {
        LoadLibraryExW(
            windows_sys::w!("pifmgr.dll"),
            null(),
            LOAD_LIBRARY_AS_DATAFILE
                | LOAD_LIBRARY_AS_IMAGE_RESOURCE
                | LOAD_LIBRARY_SEARCH_SYSTEM32,
        )
    };
    let icon = unsafe {
        LoadImageW(
            icon_lib,
            20_u16 as usize as *const _,
            IMAGE_ICON,
            32,
            32,
            LR_DEFAULTCOLOR,
        )
    };
    let wc = WNDCLASSEXW {
        cbSize: size_of::<WNDCLASSEXW>() as u32,
        style: 0,
        lpfnWndProc: Some(windows_proc),
        cbClsExtra: 0,
        cbWndExtra: 0,
        hInstance: instance,
        hIcon: icon,
        hCursor: unsafe { LoadCursorW(null(), IDC_ARROW) },
        hbrBackground: (COLOR_WINDOWFRAME) as *mut c_void,
        lpszMenuName: null(),
        lpszClassName: windows_sys::w!("temp_pad_window_class"),
        hIconSm: icon,
    };
    let class_atom = unsafe { RegisterClassExW(&wc) };
    if class_atom == 0 {
        return 1;
    }
    let hwnd = unsafe {
        CreateWindowExW(
            WS_EX_CLIENTEDGE,
            class_atom as usize as *const u16,
            windows_sys::w!("Temp Pad"),
            WS_OVERLAPPEDWINDOW,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            1000,
            600,
            null(),
            null(),
            instance,
            null(),
        )
    };
    if hwnd.is_null() {
        return 1;
    }
    unsafe { ShowWindow(hwnd, SW_SHOWDEFAULT) };
    unsafe { UpdateWindow(hwnd) };

    let mut msg = unsafe { mem::zeroed() };
    while unsafe { GetMessageW(&mut msg, null(), 0, 0) } > 0 {
        // Select all when ctrl-a is pressed
        if msg.message == WM_CHAR && msg.wParam == CONTROL_A {
            unsafe { SendMessageW(EDIT_CONTROL.load(Ordering::Acquire), EM_SETSEL, 0, -1) };
        }
        unsafe { TranslateMessage(&msg) };
        unsafe { DispatchMessageW(&msg) };
    }
    msg.wParam as i32
}

unsafe extern "system" fn windows_proc(
    hwnd: HWND,
    msg: u32,
    w_param: WPARAM,
    l_param: LPARAM,
) -> LRESULT {
    match msg {
        WM_CREATE => unsafe {
            let edit = CreateWindowExW(
                0,
                windows_sys::w!("EDIT"),
                null(),
                WS_CHILD
                    | WS_VISIBLE
                    | WS_VSCROLL
                    | WS_HSCROLL
                    | ES_LEFT as u32
                    | ES_MULTILINE as u32
                    | ES_AUTOHSCROLL as u32
                    | ES_NOHIDESEL as u32,
                0,
                0,
                0,
                0,
                hwnd,
                ID_EDITCHILD,
                GetWindowLongPtrW(hwnd, GWLP_HINSTANCE) as *mut c_void,
                null(),
            );
            EDIT_CONTROL.store(edit, Ordering::Release);

            // Attempt to set the font to Consolas
            let font = CreateFontW(
                16,
                0,
                0,
                0,
                FW_DONTCARE as i32,
                false as u32,
                false as u32,
                false as u32,
                ANSI_CHARSET as u32,
                OUT_TT_PRECIS as u32,
                CLIP_DEFAULT_PRECIS as u32,
                DEFAULT_QUALITY as u32,
                (DEFAULT_PITCH | FF_DONTCARE) as u32,
                windows_sys::w!("Consolas"),
            );
            SendMessageW(edit, WM_SETFONT, font as usize, TRUE as isize);

            // Set the tab stop to four spaces
            let mut size = mem::zeroed();
            let dc = GetDC(edit);
            GetTextExtentPoint32W(dc, windows_sys::w!("    "), 4, &mut size);
            ReleaseDC(edit, dc);
            SendMessageW(edit, EM_SETTABSTOPS, 1, addr_of!(size.cx) as isize);
        },
        WM_SETFOCUS => unsafe {
            SetFocus(EDIT_CONTROL.load(Ordering::Acquire));
        },
        WM_SIZE => unsafe {
            let edit = EDIT_CONTROL.load(Ordering::Acquire);
            MoveWindow(
                edit,
                0,
                0,
                (l_param & 0xFFFF) as i32,
                ((l_param >> 16) & 0xFFFF) as i32,
                TRUE,
            );
            let mut rect = mem::zeroed();
            GetClientRect(edit, &mut rect);
            InflateRect(&mut rect, -16, -16);
            SendMessageW(edit, EM_SETRECT, 0, addr_of!(rect) as isize);
        },
        WM_CLOSE => {
            DestroyWindow(hwnd);
        }
        WM_DESTROY => PostQuitMessage(0),
        _ => return DefWindowProcW(hwnd, msg, w_param, l_param),
    }
    0
}

#[cfg(not(test))]
#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    // TODO: abort
    loop {}
}
