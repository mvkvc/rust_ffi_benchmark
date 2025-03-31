type
  Numbers = object
    a: cint
    b: cint

proc add*(a: cint, b: cint): cint {.exportc, dynlib.} =
  a + b

proc add_struct*(nums: Numbers): cint {.exportc, dynlib.} =
  nums.a + nums.b
