%struct.__sFILE = type { i8*, i32, i32, i16, i16, %struct.__sbuf, i32, i8*, i32 (i8*)*, i32 (i8*, i8*, i32)*, i64 (i8*, i64, i32)*, i32 (i8*, i8*, i32)*, %struct.__sbuf, %struct.__sFILEX*, i32, [3 x i8], [1 x i8], %struct.__sbuf, i32, i64 }
%struct.__sFILEX = type opaque
%struct.__sbuf = type { i8*, i32 }

@.str = private unnamed_addr constant [4 x i8] c"%d\0A\00", align 1
@.str.1 = private unnamed_addr constant [14 x i8] c"runtime error\00", align 1
@.str.2 = private unnamed_addr constant [3 x i8] c"%d\00", align 1
@__stdinp = external global %struct.__sFILE*, align 8

; Function Attrs: nounwind ssp uwtable
define void @printInt(i32) #0 {
  %2 = alloca i32, align 4
  store i32 %0, i32* %2, align 4
  %3 = load i32, i32* %2, align 4
  %4 = call i32 (i8*, ...) @printf(i8* getelementptr inbounds ([4 x i8], [4 x i8]* @.str, i32 0, i32 0), i32 %3)
  ret void
}

declare i32 @printf(i8*, ...) #1

declare i32 @puts(i8*) #1

; Function Attrs: nounwind ssp uwtable
define void @error() #0 {
  %1 = call i32 @puts(i8* getelementptr inbounds ([14 x i8], [14 x i8]* @.str.1, i32 0, i32 0))
  call void @exit(i32 1) #5
  unreachable
                                                  ; No predecessors!
  ret void
}

; Function Attrs: noreturn
declare void @exit(i32) #2

; Function Attrs: nounwind ssp uwtable
define i32 @readInt() #0 {
  %1 = alloca i32, align 4
  %2 = call i32 (i8*, ...) @scanf(i8* getelementptr inbounds ([3 x i8], [3 x i8]* @.str.2, i32 0, i32 0), i32* %1)
  %3 = call i32 @getchar()
  %4 = load i32, i32* %1, align 4
  ret i32 %4
}

declare i32 @scanf(i8*, ...) #1

declare i32 @getchar() #1

; Function Attrs: nounwind ssp uwtable
define i8* @._readString() #0 {
  %1 = alloca i8*, align 8
  %2 = alloca i8*, align 8
  %3 = alloca i64, align 8
  store i8* null, i8** %2, align 8
  store i64 0, i64* %3, align 8
  %4 = load %struct.__sFILE*, %struct.__sFILE** @__stdinp, align 8
  %5 = call i64 @getline(i8** %2, i64* %3, %struct.__sFILE* %4)
  %6 = load i8*, i8** %2, align 8
  %7 = icmp eq i8* %6, null
  br i1 %7, label %12, label %8

; <label>:8                                       ; preds = %0
  %9 = load i8*, i8** %2, align 8
  %10 = call i64 @strlen(i8* %9)
  %11 = icmp eq i64 %10, 0
  br i1 %11, label %12, label %25

; <label>:12                                      ; preds = %8, %0
  %13 = load i8*, i8** %2, align 8
  %14 = icmp eq i8* %13, null
  br i1 %14, label %15, label %17

; <label>:15                                      ; preds = %12
  %16 = call i8* @malloc(i64 1)
  store i8* %16, i8** %2, align 8
  br label %17

; <label>:17                                      ; preds = %15, %12
  %18 = load i8*, i8** %2, align 8
  %19 = icmp ne i8* %18, null
  br i1 %19, label %20, label %23

; <label>:20                                      ; preds = %17
  %21 = load i8*, i8** %2, align 8
  %22 = getelementptr inbounds i8, i8* %21, i64 0
  store i8 0, i8* %22, align 1
  br label %23

; <label>:23                                      ; preds = %20, %17
  %24 = load i8*, i8** %2, align 8
  store i8* %24, i8** %1, align 8
  br label %42

; <label>:25                                      ; preds = %8
  %26 = load i8*, i8** %2, align 8
  %27 = call i64 @strlen(i8* %26)
  store i64 %27, i64* %3, align 8
  %28 = load i64, i64* %3, align 8
  %29 = sub i64 %28, 1
  %30 = load i8*, i8** %2, align 8
  %31 = getelementptr inbounds i8, i8* %30, i64 %29
  %32 = load i8, i8* %31, align 1
  %33 = sext i8 %32 to i32
  %34 = icmp eq i32 %33, 10
  br i1 %34, label %35, label %40

; <label>:35                                      ; preds = %25
  %36 = load i64, i64* %3, align 8
  %37 = sub i64 %36, 1
  %38 = load i8*, i8** %2, align 8
  %39 = getelementptr inbounds i8, i8* %38, i64 %37
  store i8 0, i8* %39, align 1
  br label %40

; <label>:40                                      ; preds = %35, %25
  %41 = load i8*, i8** %2, align 8
  store i8* %41, i8** %1, align 8
  br label %42

; <label>:42                                      ; preds = %40, %23
  %43 = load i8*, i8** %1, align 8
  ret i8* %43
}

declare i64 @getline(i8**, i64*, %struct.__sFILE*) #1

declare i64 @strlen(i8*) #1

declare i8* @malloc(i64) #1
declare void @free(i8*)

; Function Attrs: nounwind ssp uwtable
define i8* @._raw_concatenate(i8*, i8*) #0 {
  %3 = alloca i8*, align 8
  %4 = alloca i8*, align 8
  %5 = alloca i8*, align 8
  %6 = alloca i64, align 8
  %7 = alloca i8*, align 8
  store i8* %0, i8** %4, align 8
  store i8* %1, i8** %5, align 8
  %8 = load i8*, i8** %4, align 8
  %9 = call i64 @strlen(i8* %8)
  store i64 %9, i64* %6, align 8
  %10 = load i64, i64* %6, align 8
  %11 = load i8*, i8** %5, align 8
  %12 = call i64 @strlen(i8* %11)
  %13 = add i64 %10, %12
  %14 = add i64 %13, 1
  %15 = call i8* @malloc(i64 %14)
  store i8* %15, i8** %7, align 8
  %16 = load i8*, i8** %7, align 8
  %17 = icmp eq i8* %16, null
  br i1 %17, label %18, label %19

; <label>:18                                      ; preds = %2
  store i8* null, i8** %3, align 8
  br label %35

; <label>:19                                      ; preds = %2
  %20 = load i8*, i8** %7, align 8
  %21 = load i8*, i8** %4, align 8
  %22 = load i8*, i8** %7, align 8
  %23 = call i64 @llvm.objectsize.i64.p0i8(i8* %22, i1 false)
  %24 = call i8* @__strcpy_chk(i8* %20, i8* %21, i64 %23) #6
  %25 = load i8*, i8** %7, align 8
  %26 = load i64, i64* %6, align 8
  %27 = getelementptr inbounds i8, i8* %25, i64 %26
  %28 = load i8*, i8** %5, align 8
  %29 = load i8*, i8** %7, align 8
  %30 = load i64, i64* %6, align 8
  %31 = getelementptr inbounds i8, i8* %29, i64 %30
  %32 = call i64 @llvm.objectsize.i64.p0i8(i8* %31, i1 false)
  %33 = call i8* @__strcpy_chk(i8* %27, i8* %28, i64 %32) #6
  %34 = load i8*, i8** %7, align 8
  store i8* %34, i8** %3, align 8
  br label %35

; <label>:35                                      ; preds = %19, %18
  %36 = load i8*, i8** %3, align 8
  ret i8* %36
}

define void @printString({ i32, i8* }* %s) {
  %s_val = load { i32, i8* }, { i32, i8* }* %s
  %str = extractvalue { i32, i8* } %s_val, 1
  call i32 @puts(i8* %str)
  ret void
}

define { i32, i8* }* @readString() {
  %str = call i8* @._readString()

  %sizeof_tmp = getelementptr { i32, i8* }, { i32, i8* }* null, i32 1
  %sizeof = ptrtoint { i32, i8* }* %sizeof_tmp to i64
  %struct_ptr_tmp = call i8* @malloc(i64 %sizeof)
  %struct_ptr = bitcast i8* %struct_ptr_tmp to { i32, i8* }*

  %struct_tmp = insertvalue { i32, i8* } undef, i32 1, 0
  %struct = insertvalue { i32, i8* } %struct_tmp, i8* %str, 1
  store { i32, i8* } %struct, { i32, i8* }* %struct_ptr
  ret { i32, i8* }* %struct_ptr
}

%string_t = type { i32, i8*, i1 }

define %string_t* @._concatenate(%string_t* %r_1, %string_t* %r_2) {
; Getting lhs string
	%lhs_str_ptr = getelementptr %string_t, %string_t* %r_1, i32 0, i32 1
	%lhs_str = load i8*, i8** %lhs_str_ptr
; Getting rhs string
	%rhs_str_ptr = getelementptr %string_t, %string_t* %r_2, i32 0, i32 1
	%rhs_str = load i8*, i8** %rhs_str_ptr
; Concatenate
	%res_str = call i8* @._raw_concatenate(i8* %lhs_str, i8* %rhs_str)
	%res_struct = call %string_t* @._alloc_str()
  call void @._retain_str(%string_t* %res_struct)
	%res_str_ptr = getelementptr %string_t, %string_t* %res_struct, i32 0, i32 1
	store i8* %res_str, i8** %res_str_ptr
	ret %string_t* %res_struct
}

define void @._init_str_arr({ i32, %string_t**}* %arr_ptr) {
  %arr_val = load { i32, %string_t** }, { i32, %string_t** }* %arr_ptr
  %size = extractvalue { i32, %string_t** } %arr_val, 0
  %is_empty = icmp sle i32 %size, 0
  br i1 %is_empty, label %end, label %start

start:
  %str_arr = extractvalue { i32, %string_t** } %arr_val, 1

  ; create empty string
  %sizeof_tmp = getelementptr %string_t, %string_t* null, i32 1
  %sizeof = ptrtoint %string_t* %sizeof_tmp to i64
  %struct_ptr_tmp = call i8* @malloc(i64 %sizeof)
  %struct_ptr = bitcast i8* %struct_ptr_tmp to %string_t*

  %struct_tmp = insertvalue %string_t undef, i32 %size, 0
  %struct_tmp2 = insertvalue %string_t %struct_tmp, i8* null, 1
  %struct = insertvalue %string_t %struct_tmp2, i1 false, 2
  store %string_t %struct, %string_t* %struct_ptr
  br label %loop_body

loop_body:
  %idx = phi i32 [0, %start], [%next_idx, %loop_body]
  %elem_ptr = getelementptr %string_t*, %string_t** %str_arr, i32 %idx
  store %string_t* %struct_ptr, %string_t** %elem_ptr

  %next_idx = add i32 %idx, 1
  %is_last = icmp eq i32 %next_idx, %size
  br i1 %is_last, label %end, label %loop_body

end:
  ret void
}

define %string_t* @._alloc_str() {
	%r_1 = getelementptr %string_t, %string_t* null, i32 1
	%r_2 = ptrtoint %string_t* %r_1 to i64
	%r_3 = call i8* @malloc(i64 %r_2)
	%r_4 = bitcast i8* %r_3 to %string_t*
	%r_5 = insertvalue %string_t undef, i32 0, 0
	%r_6 = insertvalue %string_t %r_5, i8* null, 1
	%r_7 = insertvalue %string_t %r_6, i1 false, 2
	store %string_t %r_7, %string_t* %r_4
  ret %string_t* %r_4
}

define void @._retain_str(%string_t* %s) {
  %s_val = load %string_t, %string_t* %s
  %refs = extractvalue %string_t %s_val, 0
  %new_refs = add i32 %refs, 1
  %s_new_val = insertvalue %string_t %s_val, i32 %new_refs, 0
  store %string_t %s_new_val, %string_t* %s
  ret void
}

define void @._release_str(%string_t* %s) {
  %s_val = load %string_t, %string_t* %s
  %refs = extractvalue %string_t %s_val, 0
  %new_refs = sub i32 %refs, 1
  %to_free = icmp eq i32 %new_refs, 0
  br i1 %to_free, label %del_struct, label %update

del_struct:
  %struct_ptr = bitcast %string_t* %s to i8*
  call void @printInt(i32 55)
  call void @free(i8* %struct_ptr)
  %is_const = extractvalue %string_t %s_val, 2
  br i1 %is_const, label %end, label %del_ptr

del_ptr:
  call void @printInt(i32 77)
  %s_ptr = extractvalue %string_t %s_val, 1
  call void @free(i8* %s_ptr)
  br label %end

update:
  %s_new_val = insertvalue %string_t %s_val, i32 %new_refs, 0
  store %string_t %s_new_val, %string_t* %s
  br label %end

end:
  ret void
}

; Function Attrs: nounwind
declare i8* @__strcpy_chk(i8*, i8*, i64) #3

; Function Attrs: nounwind readnone
declare i64 @llvm.objectsize.i64.p0i8(i8*, i1) #4

attributes #0 = { nounwind ssp uwtable "disable-tail-calls"="false" "less-precise-fpmad"="false" "no-frame-pointer-elim"="true" "no-frame-pointer-elim-non-leaf" "no-infs-fp-math"="false" "no-nans-fp-math"="false" "stack-protector-buffer-size"="8" "target-cpu"="penryn" "target-features"="+cx16,+mmx,+sse,+sse2,+sse3,+sse4.1,+ssse3" "unsafe-fp-math"="false" "use-soft-float"="false" }
attributes #1 = { "disable-tail-calls"="false" "less-precise-fpmad"="false" "no-frame-pointer-elim"="true" "no-frame-pointer-elim-non-leaf" "no-infs-fp-math"="false" "no-nans-fp-math"="false" "stack-protector-buffer-size"="8" "target-cpu"="penryn" "target-features"="+cx16,+mmx,+sse,+sse2,+sse3,+sse4.1,+ssse3" "unsafe-fp-math"="false" "use-soft-float"="false" }
attributes #2 = { noreturn "disable-tail-calls"="false" "less-precise-fpmad"="false" "no-frame-pointer-elim"="true" "no-frame-pointer-elim-non-leaf" "no-infs-fp-math"="false" "no-nans-fp-math"="false" "stack-protector-buffer-size"="8" "target-cpu"="penryn" "target-features"="+cx16,+mmx,+sse,+sse2,+sse3,+sse4.1,+ssse3" "unsafe-fp-math"="false" "use-soft-float"="false" }
attributes #3 = { nounwind "disable-tail-calls"="false" "less-precise-fpmad"="false" "no-frame-pointer-elim"="true" "no-frame-pointer-elim-non-leaf" "no-infs-fp-math"="false" "no-nans-fp-math"="false" "stack-protector-buffer-size"="8" "target-cpu"="penryn" "target-features"="+cx16,+mmx,+sse,+sse2,+sse3,+sse4.1,+ssse3" "unsafe-fp-math"="false" "use-soft-float"="false" }
attributes #4 = { nounwind readnone }
attributes #5 = { noreturn }
attributes #6 = { nounwind }

!llvm.module.flags = !{!0}
!llvm.ident = !{!1}

!0 = !{i32 1, !"PIC Level", i32 2}
!1 = !{!"Apple LLVM version 8.0.0 (clang-800.0.42.1)"}
