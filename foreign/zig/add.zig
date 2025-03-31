const std = @import("std");

pub const Numbers = extern struct {
    a: i32,
    b: i32,
};

export fn add(a: i32, b: i32) i32 {
    return a + b;
}

export fn add_struct(nums: Numbers) i32 {
    return nums.a + nums.b;
}