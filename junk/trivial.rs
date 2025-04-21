use core::ffi::c_int;
use std::ptr;

#[repr(C)]
pub struct School {
    district: c_int,
    num_teachers: c_int,
}

#[repr(C)]
pub struct Student {
    school: *mut School,
    age: c_int,
    grade: c_int,
}

// AI Seyz: Implement Drop to handle deallocation of the school pointer
impl Drop for Student {
    fn drop(&mut self) {
        if !self.school.is_null() {
            let _ = unsafe { Box::from_raw(self.school) };
        }
    }
}

#[no_mangle]
pub extern "C" fn create_student(
    age: c_int,
    grade: c_int,
    district: c_int,
    num_teachers: c_int,
) -> *mut Student {
    if age < 0 {
        return ptr::null_mut();
    }
    Box::into_raw(Box::new(Student {
        school: Box::into_raw(Box::new(School {
            district,
            num_teachers,
        })),
        age,
        grade,
    }))
}

#[no_mangle]
pub extern "C" fn dealloc_student(student: *mut Student) {
    if !student.is_null() {
        let _ = unsafe { Box::from_raw(student) };
        // Drop impl handles freeing the school
    }
}

#[no_mangle]
pub extern "C" fn increase_student_age(student: *mut Student) -> c_int {
    if student.is_null() {
        return -1;
    }
    let student = unsafe { &mut *student };
    student.age += 1;
    student.age
}

#[no_mangle]
pub extern "C" fn get_student_grade(student: *const Student) -> c_int {
    if student.is_null() {
        return -1;
        // panic!("null student");
    }
    unsafe { &*student }.grade
}

#[no_mangle]
pub extern "C" fn display_student(student: *const Student) {
    if student.is_null() {
        println!("student is empty");
        return;
    }
    let student = unsafe { &*student };
    println!("student: age: {} grade: {}", student.age, student.grade);
    if student.school.is_null() {
        println!("  school: empty");
    } else {
        let school = unsafe { &*student.school };
        println!(
            "  school: district: {} num_teachers: {}",
            school.district, school.num_teachers
        );
    }
}

#[no_mangle]
pub extern "C" fn safe_create_student(
    age: i32,
    grade: i32,
    district: i32,
    num_teachers: i32,
) -> Option<Box<Student>> {
    if age < 0 {
        return None;
    }
    Some(Box::new(Student {
        school: Box::into_raw(Box::new(School {
            district,
            num_teachers,
        })),
        age,
        grade,
    }))
}

#[no_mangle]
pub extern "C" fn safe_increase_student_age(student: &mut Student) -> i32 {
    student.age += 1;
    student.age
}

#[no_mangle]
pub extern "C" fn safe_get_student_grade(student: &Student) -> i32 {
    student.grade
}

#[no_mangle]
pub extern "C" fn safe_get_student_grade_ptr(student_ptr: *const Student) -> Option<i32> {
    if student_ptr.is_null() {
        None
    } else {
        Some(unsafe { &*student_ptr }.grade)
    }
}

#[no_mangle]
pub extern "C" fn safe_display_student(student: &Student) {
    println!("student: age: {} grade: {}", student.age, student.grade);
    if student.school.is_null() {
        println!("  school: empty");
    } else {
        let school = unsafe { &*student.school };
        println!(
            "  school: district: {} num_teachers: {}",
            school.district, school.num_teachers
        );
    }
}
