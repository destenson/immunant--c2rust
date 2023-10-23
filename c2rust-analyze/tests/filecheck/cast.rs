// CHECK-DAG: struct S<'h0> {
struct S {
    // CHECK-DAG: i: &'h0 i32
    i: *const i32,
}

// The below ensures the concrete origins for `s` and `s.i` are the same and are hypothetical
// CHECK-DAG: assign {{.*}}#*mut S{{.*}}origin_params: [('h0, Origin([[HYPO_ORIGIN:[0-9]+]]))]{{.*}} = Label{{.*}}origin_params: [('h0, Origin({{.*}}))]
// CHECK-DAG: assign Label { origin: Some(Origin([[HYPO_ORIGIN]])){{.*}}*const i32{{.*}} = Label

// CHECK-LABEL: final labeling for "null_ptr"
pub unsafe fn null_ptr() {
    // CHECK-DAG: ([[@LINE+5]]: s): addr_of = UNIQUE, type = READ | WRITE | UNIQUE#
    // CHECK-LABEL: type assignment for "null_ptr":
    // FIXME: regression: bad type rewrite; should be:
    // XXXXX-DAG: ([[@LINE+1]]: s): &mut S
    // CHECK-DAG: ([[@LINE+1]]: s): *mut S
    let s = 0 as *mut S;
    (*s).i = 0 as *const i32;
}

#[repr(C)]
pub struct Foo {
    y: *mut i32,
}

extern "C" {
    // necessary to fix the type of Foo::y
    fn bar(f: Foo);
}

// CHECK-LABEL: pub unsafe fn cell_as_mut_as_cell<'h0>(mut x: &'h0 core::cell::Cell<(i32)>, mut f: Foo) {
pub unsafe fn cell_as_mut_as_cell(mut x: *mut i32, mut f: Foo) {
    let z = x;
    let r = x;
    *z = 1;
    *r = 1;
    *z = 4;
    // CHECK-DAG: f.y = (x).as_ptr();
    f.y = x;
    // CHECK-DAG: x = &*((f.y) as *const std::cell::Cell<i32>);
    x = f.y;
}
pub struct fdnode {
    pub ctx: *mut u8,
}

// CHECK-LABEL: unsafe extern "C" fn server_free<'h0,'h1>(fdn: &'h0 (fdnode<'h1>)) {
unsafe extern "C" fn server_free(fdn: *mut fdnode) {
    let _fdn2 = fdn as *const fdnode;
}
