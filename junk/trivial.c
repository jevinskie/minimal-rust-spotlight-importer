#include <assert.h>
#include <stdio.h>
#include <stdlib.h>

struct school_s;
typedef struct school_s school_t;

typedef struct student_s {
    school_t *school;
    int age;
    int grade;
} student_t;

typedef struct school_s {
    int district;
    int num_teachers;
} school_t;

student_t *create_student(int age, int grade, int district, int num_teachers) {
    if (age < 0) {
        return NULL;
    }
    student_t *student = malloc(sizeof(student_t));
    if (!student) {
        return NULL;
    }
    school_t *school = malloc(sizeof(school_t));
    if (!school) {
        free(student);
        return NULL;
    }
    school->district     = district;
    school->num_teachers = num_teachers;
    student->school      = school;
    student->age         = age;
    student->grade       = grade;
    return student;
}

void dealloc_student(student_t *student) {
    if (student) {
        if (student->school) {
            free(student->school);
        }
        free(student);
    }
}

int increase_student_age(student_t *student) {
    if (!student) {
        return -1;
    }
    const int new_age = student->age + 1;
    student->age      = new_age;
    return new_age;
}

void display_student(student_t *const student) {
    if (!student) {
        printf("student is empty\n");
        return;
    }
    printf("student: age: %d grade: %d\n", student->age, student->grade);
    if (!student->school) {
        printf("  school: empty\n");
    } else {
        printf("  school: district: %d num_teachers: %d\n", student->school->district,
               student->school->num_teachers);
    }
}
