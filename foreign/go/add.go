package main

/*
#include "../_include/numbers.h"
*/
import "C"

//export add
func add(a, b C.int) C.int {
	return a + b
}

//export add_struct
func add_struct(nums C.struct_Numbers) C.int {
	return nums.a + nums.b
}

func main() {}
