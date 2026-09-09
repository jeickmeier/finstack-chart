	.build_version macos, 26, 0	sdk_version 26, 5
	.section	__TEXT,__text,regular,pure_instructions
	.globl	__Z18__ieee754_rem_pio2dPd      ; -- Begin function _Z18__ieee754_rem_pio2dPd
	.p2align	2
__Z18__ieee754_rem_pio2dPd:             ; @_Z18__ieee754_rem_pio2dPd
	.cfi_startproc
; %bb.0:
	sub	sp, sp, #64
	stp	x20, x19, [sp, #32]             ; 16-byte Folded Spill
	stp	x29, x30, [sp, #48]             ; 16-byte Folded Spill
	add	x29, sp, #48
	.cfi_def_cfa w29, 16
	.cfi_offset w30, -8
	.cfi_offset w29, -16
	.cfi_offset w19, -24
	.cfi_offset w20, -32
Lloh0:
	adrp	x8, ___stack_chk_guard@GOTPAGE
Lloh1:
	ldr	x8, [x8, ___stack_chk_guard@GOTPAGEOFF]
Lloh2:
	ldr	x8, [x8]
	str	x8, [sp, #24]
	fmov	x20, d0
	ubfx	x9, x20, #32, #31
	mov	w8, #8699                       ; =0x21fb
	movk	w8, #16361, lsl #16
	cmp	w9, w8
	b.hi	LBB0_2
; %bb.1:
	mov	w8, #0                          ; =0x0
	str	d0, [x0]
	str	xzr, [x0, #8]
	b	LBB0_29
LBB0_2:
	mov	w8, #55675                      ; =0xd97b
	movk	w8, #16386, lsl #16
	cmp	w9, w8
	b.hi	LBB0_6
; %bb.3:
	lsr	x8, x20, #32
	cmp	w8, #1
	b.lt	LBB0_15
; %bb.4:
	mov	x8, #1413480448                 ; =0x54400000
	movk	x8, #8699, lsl #32
	movk	x8, #49145, lsl #48
	fmov	d1, x8
	fadd	d0, d0, d1
	mov	w8, #8699                       ; =0x21fb
	movk	w8, #16377, lsl #16
	cmp	w9, w8
	b.ne	LBB0_17
; %bb.5:
	mov	x8, #442499072                  ; =0x1a600000
	movk	x8, #46177, lsl #32
	movk	x8, #48592, lsl #48
	fmov	d1, x8
	fadd	d0, d0, d1
	mov	x8, #28787                      ; =0x7073
	movk	x8, #11779, lsl #16
	movk	x8, #6538, lsl #32
	movk	x8, #48035, lsl #48
	b	LBB0_18
LBB0_6:
	mov	w8, #8699                       ; =0x21fb
	movk	w8, #16697, lsl #16
	cmp	w9, w8
	b.hi	LBB0_13
; %bb.7:
	fabs	d0, d0
	mov	x8, #51331                      ; =0xc883
	movk	x8, #28105, lsl #16
	movk	x8, #24368, lsl #32
	movk	x8, #16356, lsl #48
	fmov	d1, x8
	fmov	d2, #0.50000000
	fmadd	d1, d0, d1, d2
	fcvtzs	w8, d1
	scvtf	d3, w8
	mov	x10, #1413480448                ; =0x54400000
	movk	x10, #8699, lsl #32
	movk	x10, #49145, lsl #48
	fmov	d1, x10
	fmadd	d0, d3, d1, d0
	mov	x10, #25393                     ; =0x6331
	movk	x10, #6754, lsl #16
	movk	x10, #46177, lsl #32
	movk	x10, #15824, lsl #48
	fmov	d1, x10
	fmul	d1, d3, d1
	cmp	w8, #31
	b.gt	LBB0_9
; %bb.8:
Lloh3:
	adrp	x10, __ZZ18__ieee754_rem_pio2dPdE8npio2_hw@PAGE
Lloh4:
	add	x10, x10, __ZZ18__ieee754_rem_pio2dPdE8npio2_hw@PAGEOFF
	add	x10, x10, w8, sxtw #2
	ldur	w10, [x10, #-4]
	cmp	w9, w10
	b.ne	LBB0_12
LBB0_9:
	lsr	w9, w9, #20
	fsub	d2, d0, d1
	str	d2, [x0]
	fmov	x10, d2
	ubfx	x10, x10, #52, #11
	sub	w10, w9, w10
	cmp	w10, #17
	b.lt	LBB0_26
; %bb.10:
	mov	x10, #442499072                 ; =0x1a600000
	movk	x10, #46177, lsl #32
	movk	x10, #15824, lsl #48
	fmov	d1, x10
	fmsub	d4, d3, d1, d0
	fsub	d0, d0, d4
	fmsub	d0, d3, d1, d0
	mov	x10, #28787                     ; =0x7073
	movk	x10, #11779, lsl #16
	movk	x10, #6538, lsl #32
	movk	x10, #15267, lsl #48
	fmov	d1, x10
	fnmsub	d1, d3, d1, d0
	fsub	d2, d4, d1
	str	d2, [x0]
	fmov	x10, d2
	ubfx	x10, x10, #52, #11
	sub	w9, w9, w10
	cmp	w9, #50
	b.lt	LBB0_25
; %bb.11:
	mov	x9, #771751936                  ; =0x2e000000
	movk	x9, #6538, lsl #32
	movk	x9, #15267, lsl #48
	fmov	d1, x9
	fmsub	d0, d3, d1, d4
	fsub	d2, d4, d0
	fmsub	d1, d3, d1, d2
	mov	x9, #18881                      ; =0x49c1
	movk	x9, #9504, lsl #16
	movk	x9, #33690, lsl #32
	movk	x9, #14715, lsl #48
	fmov	d2, x9
	fnmsub	d1, d3, d2, d1
LBB0_12:
	fsub	d2, d0, d1
	str	d2, [x0]
	b	LBB0_26
LBB0_13:
	lsr	w8, w9, #20
	cmp	w8, #2047
	b.lo	LBB0_19
; %bb.14:
	mov	w8, #0                          ; =0x0
	fsub	d0, d0, d0
	stp	d0, d0, [x0]
	b	LBB0_29
LBB0_15:
	mov	x8, #1413480448                 ; =0x54400000
	movk	x8, #8699, lsl #32
	movk	x8, #16377, lsl #48
	fmov	d1, x8
	fadd	d0, d0, d1
	mov	w8, #8699                       ; =0x21fb
	movk	w8, #16377, lsl #16
	cmp	w9, w8
	b.ne	LBB0_23
; %bb.16:
	mov	x8, #442499072                  ; =0x1a600000
	movk	x8, #46177, lsl #32
	movk	x8, #15824, lsl #48
	fmov	d1, x8
	fadd	d0, d0, d1
	mov	x8, #28787                      ; =0x7073
	movk	x8, #11779, lsl #16
	movk	x8, #6538, lsl #32
	movk	x8, #15267, lsl #48
	b	LBB0_24
LBB0_17:
	mov	x8, #25393                      ; =0x6331
	movk	x8, #6754, lsl #16
	movk	x8, #46177, lsl #32
	movk	x8, #48592, lsl #48
LBB0_18:
	fmov	d1, x8
	fadd	d2, d0, d1
	fsub	d0, d0, d2
	fadd	d0, d0, d1
	stp	d2, d0, [x0]
	mov	w8, #1                          ; =0x1
	b	LBB0_29
LBB0_19:
	mov	x19, x0
	sub	w2, w8, #1046
	sub	w8, w9, w2, lsl #20
	mov	x9, x20
	bfi	x9, x8, #32, #32
	fmov	d0, x9
	fcvtzs	w8, d0
	scvtf	d1, w8
	fsub	d0, d0, d1
	fcvtzs	w8, d0, #24
	scvtf	d2, w8
	stp	d1, d2, [sp]
	mov	x8, #4715268809856909312        ; =0x4170000000000000
	fmov	d1, x8
	fnmsub	d0, d0, d1, d2
	fmul	d0, d0, d1
	str	d0, [sp, #16]
	mov	w8, #2                          ; =0x2
	mov	x9, sp
LBB0_20:                                ; =>This Inner Loop Header: Depth=1
	ldr	d0, [x9, x8, lsl #3]
	sub	x8, x8, #1
	fcmp	d0, #0.0
	b.eq	LBB0_20
; %bb.21:
Lloh5:
	adrp	x5, __ZZ18__ieee754_rem_pio2dPdE11two_over_pi@PAGE
Lloh6:
	add	x5, x5, __ZZ18__ieee754_rem_pio2dPdE11two_over_pi@PAGEOFF
	mov	x0, sp
	add	w3, w8, #2
	mov	x1, x19
	mov	w4, #2                          ; =0x2
	bl	__Z17__kernel_rem_pio2PdS_iiiPKi
	mov	x8, x0
	tbz	x20, #63, LBB0_29
; %bb.22:
	ldr	q0, [x19]
	fneg.2d	v0, v0
	str	q0, [x19]
	b	LBB0_28
LBB0_23:
	mov	x8, #25393                      ; =0x6331
	movk	x8, #6754, lsl #16
	movk	x8, #46177, lsl #32
	movk	x8, #15824, lsl #48
LBB0_24:
	fmov	d1, x8
	fadd	d2, d0, d1
	fsub	d0, d0, d2
	fadd	d0, d0, d1
	stp	d2, d0, [x0]
	mov	w8, #-1                         ; =0xffffffff
	b	LBB0_29
LBB0_25:
	mov.16b	v0, v4
LBB0_26:
	fsub	d0, d0, d2
	fsub	d0, d0, d1
	str	d0, [x0, #8]
	tbz	x20, #63, LBB0_29
; %bb.27:
	fneg	d1, d2
	fneg	d0, d0
	stp	d1, d0, [x0]
LBB0_28:
	neg	w8, w8
LBB0_29:
	ldr	x9, [sp, #24]
Lloh7:
	adrp	x10, ___stack_chk_guard@GOTPAGE
Lloh8:
	ldr	x10, [x10, ___stack_chk_guard@GOTPAGEOFF]
Lloh9:
	ldr	x10, [x10]
	cmp	x10, x9
	b.ne	LBB0_31
; %bb.30:
	mov	x0, x8
	ldp	x29, x30, [sp, #48]             ; 16-byte Folded Reload
	ldp	x20, x19, [sp, #32]             ; 16-byte Folded Reload
	add	sp, sp, #64
	ret
LBB0_31:
	bl	___stack_chk_fail
	.loh AdrpLdrGotLdr	Lloh0, Lloh1, Lloh2
	.loh AdrpAdd	Lloh3, Lloh4
	.loh AdrpAdd	Lloh5, Lloh6
	.loh AdrpLdrGotLdr	Lloh7, Lloh8, Lloh9
	.cfi_endproc
                                        ; -- End function
	.globl	__Z17__kernel_rem_pio2PdS_iiiPKi ; -- Begin function _Z17__kernel_rem_pio2PdS_iiiPKi
	.p2align	2
__Z17__kernel_rem_pio2PdS_iiiPKi:       ; @_Z17__kernel_rem_pio2PdS_iiiPKi
	.cfi_startproc
; %bb.0:
	stp	d13, d12, [sp, #-144]!          ; 16-byte Folded Spill
	stp	d11, d10, [sp, #16]             ; 16-byte Folded Spill
	stp	d9, d8, [sp, #32]               ; 16-byte Folded Spill
	stp	x28, x27, [sp, #48]             ; 16-byte Folded Spill
	stp	x26, x25, [sp, #64]             ; 16-byte Folded Spill
	stp	x24, x23, [sp, #80]             ; 16-byte Folded Spill
	stp	x22, x21, [sp, #96]             ; 16-byte Folded Spill
	stp	x20, x19, [sp, #112]            ; 16-byte Folded Spill
	stp	x29, x30, [sp, #128]            ; 16-byte Folded Spill
	add	x29, sp, #128
	sub	sp, sp, #704
	.cfi_def_cfa w29, 16
	.cfi_offset w30, -8
	.cfi_offset w29, -16
	.cfi_offset w19, -24
	.cfi_offset w20, -32
	.cfi_offset w21, -40
	.cfi_offset w22, -48
	.cfi_offset w23, -56
	.cfi_offset w24, -64
	.cfi_offset w25, -72
	.cfi_offset w26, -80
	.cfi_offset w27, -88
	.cfi_offset w28, -96
	.cfi_offset b8, -104
	.cfi_offset b9, -112
	.cfi_offset b10, -120
	.cfi_offset b11, -128
	.cfi_offset b12, -136
	.cfi_offset b13, -144
	mov	x22, x3
	mov	x23, x0
Lloh10:
	adrp	x8, ___stack_chk_guard@GOTPAGE
Lloh11:
	ldr	x8, [x8, ___stack_chk_guard@GOTPAGEOFF]
Lloh12:
	ldr	x8, [x8]
	stur	x8, [x29, #-144]
Lloh13:
	adrp	x8, __ZZ17__kernel_rem_pio2PdS_iiiPKiE7init_jk@PAGE
Lloh14:
	add	x8, x8, __ZZ17__kernel_rem_pio2PdS_iiiPKiE7init_jk@PAGEOFF
	ldr	w11, [x8, w4, sxtw #2]
	sub	w9, w3, #1
	sub	w8, w2, #3
	mov	w10, #43691                     ; =0xaaab
	movk	w10, #10922, lsl #16
	smull	x8, w8, w10
	asr	x8, x8, #34
	add	w8, w8, w8, lsr #31
	bic	w8, w8, w8, asr #31
	mov	w10, #-24                       ; =0xffffffe8
	madd	w20, w8, w10, w2
	sub	w0, w20, #24
	str	x11, [sp, #112]                 ; 8-byte Folded Spill
	cmn	w11, w9
	b.mi	LBB1_6
; %bb.1:
	sub	w9, w8, w9
	ldr	x10, [sp, #112]                 ; 8-byte Folded Reload
	add	w10, w10, w22
	add	x11, sp, #448
	b	LBB1_4
LBB1_2:                                 ;   in Loop: Header=BB1_4 Depth=1
	ldr	s0, [x5, w9, uxtw #2]
	sshll.2d	v0, v0, #0
	scvtf	d0, d0
LBB1_3:                                 ;   in Loop: Header=BB1_4 Depth=1
	str	d0, [x11], #8
	add	w9, w9, #1
	subs	x10, x10, #1
	b.eq	LBB1_6
LBB1_4:                                 ; =>This Inner Loop Header: Depth=1
	tbz	w9, #31, LBB1_2
; %bb.5:                                ;   in Loop: Header=BB1_4 Depth=1
	movi.2d	v0, #0000000000000000
	b	LBB1_3
LBB1_6:
	str	x1, [sp, #16]                   ; 8-byte Folded Spill
	mov	x9, #0                          ; =0x0
	ldr	x11, [sp, #112]                 ; 8-byte Folded Reload
	bic	w10, w11, w11, asr #31
	mov	w19, w22
	sxtw	x30, w11
	add	w10, w10, #1
	sub	x11, x19, #1
	mov	w12, #-1                        ; =0xffffffff
	add	x12, x19, x12
	and	x13, x19, #0x7ffffff8
	add	x14, x23, #32
	add	x15, sp, #448
	lsr	x16, x11, #32
	add	x17, sp, #128
	mov	x7, x11
	b	LBB1_8
LBB1_7:                                 ;   in Loop: Header=BB1_8 Depth=1
	str	d0, [x17, x9, lsl #3]
	add	x9, x9, #1
	add	w7, w7, #1
	cmp	x9, x10
	b.eq	LBB1_18
LBB1_8:                                 ; =>This Loop Header: Depth=1
                                        ;     Child Loop BB1_16 Depth 2
                                        ;     Child Loop BB1_12 Depth 2
	movi.2d	v0, #0000000000000000
	cmp	w22, #1
	b.lt	LBB1_7
; %bb.9:                                ;   in Loop: Header=BB1_8 Depth=1
	cmp	w22, #8
	b.hs	LBB1_13
; %bb.10:                               ;   in Loop: Header=BB1_8 Depth=1
	mov	x1, #0                          ; =0x0
LBB1_11:                                ;   in Loop: Header=BB1_8 Depth=1
	sub	w2, w7, w1
	add	x3, x23, x1, lsl #3
	sub	x1, x19, x1
LBB1_12:                                ;   Parent Loop BB1_8 Depth=1
                                        ; =>  This Inner Loop Header: Depth=2
	ldr	d1, [x3], #8
	ldr	d2, [x15, w2, sxtw #3]
	fmadd	d0, d1, d2, d0
	sub	w2, w2, #1
	subs	x1, x1, #1
	b.ne	LBB1_12
	b	LBB1_7
LBB1_13:                                ;   in Loop: Header=BB1_8 Depth=1
	mov	x1, #0                          ; =0x0
	add	w2, w12, w9
	sub	w3, w2, w11
	cmp	w3, w2
	b.gt	LBB1_11
; %bb.14:                               ;   in Loop: Header=BB1_8 Depth=1
	cbnz	x16, LBB1_11
; %bb.15:                               ;   in Loop: Header=BB1_8 Depth=1
	mov	x1, x7
	mov	x2, x14
	mov	x3, x13
LBB1_16:                                ;   Parent Loop BB1_8 Depth=1
                                        ; =>  This Inner Loop Header: Depth=2
	ldp	q1, q2, [x2, #-32]
	ldp	q3, q4, [x2], #64
	add	x6, x15, w1, sxtw #3
	ldur	q5, [x6, #-8]
	ext.16b	v5, v5, v5, #8
	ldur	q6, [x6, #-24]
	ext.16b	v6, v6, v6, #8
	ldur	q7, [x6, #-40]
	ext.16b	v7, v7, v7, #8
	ldur	q16, [x6, #-56]
	ext.16b	v16, v16, v16, #8
	fmul.2d	v1, v1, v5
	mov	d5, v1[1]
	fmul.2d	v2, v2, v6
	mov	d6, v2[1]
	fmul.2d	v3, v3, v7
	mov	d7, v3[1]
	fmul.2d	v4, v4, v16
	mov	d16, v4[1]
	fadd	d0, d0, d1
	fadd	d0, d0, d5
	fadd	d0, d0, d2
	fadd	d0, d0, d6
	fadd	d0, d0, d3
	fadd	d0, d0, d7
	fadd	d0, d0, d4
	fadd	d0, d0, d16
	sub	w1, w1, #8
	sub	x3, x3, #8
	cbnz	x3, LBB1_16
; %bb.17:                               ;   in Loop: Header=BB1_8 Depth=1
	mov	x1, x13
	cmp	x13, x19
	b.eq	LBB1_7
	b	LBB1_11
LBB1_18:
	str	w4, [sp, #12]                   ; 4-byte Folded Spill
	mov	w9, #48                         ; =0x30
	sub	w9, w9, w20
	str	w9, [sp, #60]                   ; 4-byte Folded Spill
	mov	w9, #47                         ; =0x2f
	sub	w9, w9, w20
	str	w9, [sp, #56]                   ; 4-byte Folded Spill
	ldr	x9, [sp, #112]                  ; 8-byte Folded Reload
	cmp	w9, #1
	csinc	w10, w9, wzr, gt
	str	x10, [sp, #88]                  ; 8-byte Folded Spill
	add	w10, w10, #1
	stp	w20, w10, [sp, #48]             ; 8-byte Folded Spill
	and	x28, x19, #0x7ffffff8
	add	x26, sp, #128
	sub	x24, x26, #8
	sub	x10, x29, #224
	sub	x11, x10, #32
	str	x11, [sp, #24]                  ; 8-byte Folded Spill
	sub	x11, x10, #16
	str	x11, [sp, #32]                  ; 8-byte Folded Spill
	sub	x25, x10, #4
	add	x11, x23, #32
	fmov	d9, #0.12500000
	fmov	d10, #-8.00000000
	fmov	d11, #1.00000000
	fmov	d12, #0.50000000
	add	x27, sp, #448
	sub	x10, x27, #24
	stp	x10, x11, [sp, #72]             ; 16-byte Folded Spill
	mov	x12, #4499096027743125504       ; =0x3e70000000000000
	mov	x13, #-4508103226997866496      ; =0xc170000000000000
	mov	x11, x9
	add	x21, x5, w8, uxtw #2
	add	x8, x25, x30, lsl #2
	str	x8, [sp, #64]                   ; 8-byte Folded Spill
	str	x30, [sp, #96]                  ; 8-byte Folded Spill
	str	w0, [sp, #108]                  ; 4-byte Folded Spill
	b	LBB1_20
LBB1_19:                                ;   in Loop: Header=BB1_20 Depth=1
	mov	x11, x8
	mov	x12, #4499096027743125504       ; =0x3e70000000000000
	mov	x13, #-4508103226997866496      ; =0xc170000000000000
LBB1_20:                                ; =>This Loop Header: Depth=1
                                        ;     Child Loop BB1_22 Depth 2
                                        ;     Child Loop BB1_31 Depth 2
                                        ;     Child Loop BB1_58 Depth 2
                                        ;     Child Loop BB1_62 Depth 2
                                        ;     Child Loop BB1_65 Depth 2
                                        ;     Child Loop BB1_68 Depth 2
                                        ;     Child Loop BB1_74 Depth 2
                                        ;       Child Loop BB1_78 Depth 3
                                        ;       Child Loop BB1_81 Depth 3
	ldr	d0, [x26, w11, sxtw #3]
	cmp	w11, #1
	b.lt	LBB1_23
; %bb.21:                               ;   in Loop: Header=BB1_20 Depth=1
	mov	x8, x11
	ubfiz	x8, x8, #3, #32
	sub	x9, x29, #224
LBB1_22:                                ;   Parent Loop BB1_20 Depth=1
                                        ; =>  This Inner Loop Header: Depth=2
	fmov	d1, x12
	fmul	d1, d0, d1
	fcvtzs	w10, d1
	scvtf	d1, w10
	fmov	d2, x13
	fmadd	d0, d1, d2, d0
	fcvtzs	w10, d0
	str	w10, [x9], #4
	ldr	d0, [x24, x8]
	fadd	d0, d0, d1
	subs	x8, x8, #8
	b.ne	LBB1_22
LBB1_23:                                ;   in Loop: Header=BB1_20 Depth=1
	str	x11, [sp, #120]                 ; 8-byte Folded Spill
	sxtw	x20, w11
	bl	_scalbn
	ldr	w0, [sp, #108]                  ; 4-byte Folded Reload
	fmul	d1, d0, d9
	frintm	d1, d1
	fmadd	d0, d1, d10, d0
	fcvtzs	w13, d0
	scvtf	d1, w13
	fsub	d8, d0, d1
	cmp	w0, #1
	b.lt	LBB1_35
; %bb.24:                               ;   in Loop: Header=BB1_20 Depth=1
	sub	x8, x29, #224
	add	x8, x8, x20, lsl #2
	ldur	w9, [x8, #-4]
	ldr	w11, [sp, #60]                  ; 4-byte Folded Reload
	asr	w10, w9, w11
	add	w13, w10, w13
	lsl	w10, w10, w11
	sub	w9, w9, w10
	stur	w9, [x8, #-4]
	ldr	w8, [sp, #56]                   ; 4-byte Folded Reload
	asr	w14, w9, w8
LBB1_25:                                ;   in Loop: Header=BB1_20 Depth=1
	ldr	x2, [sp, #120]                  ; 8-byte Folded Reload
	ldr	x15, [sp, #96]                  ; 8-byte Folded Reload
	cmp	w14, #1
	b.lt	LBB1_51
; %bb.26:                               ;   in Loop: Header=BB1_20 Depth=1
	cmp	w2, #1
	b.lt	LBB1_40
LBB1_27:                                ;   in Loop: Header=BB1_20 Depth=1
	mov	w11, #0                         ; =0x0
	mov	w9, w2
	sub	x10, x29, #224
	b	LBB1_31
LBB1_28:                                ;   in Loop: Header=BB1_31 Depth=2
	mov	w11, #16777215                  ; =0xffffff
LBB1_29:                                ;   in Loop: Header=BB1_31 Depth=2
	mov	w8, #0                          ; =0x0
	sub	w11, w11, w12
	str	w11, [x10]
	mov	w11, #1                         ; =0x1
LBB1_30:                                ;   in Loop: Header=BB1_31 Depth=2
	add	x10, x10, #4
	subs	x9, x9, #1
	b.eq	LBB1_41
LBB1_31:                                ;   Parent Loop BB1_20 Depth=1
                                        ; =>  This Inner Loop Header: Depth=2
	ldr	w12, [x10]
	cbnz	w11, LBB1_28
; %bb.32:                               ;   in Loop: Header=BB1_31 Depth=2
	cbz	w12, LBB1_34
; %bb.33:                               ;   in Loop: Header=BB1_31 Depth=2
	mov	w11, #16777216                  ; =0x1000000
	b	LBB1_29
LBB1_34:                                ;   in Loop: Header=BB1_31 Depth=2
	mov	w11, #0                         ; =0x0
	mov	w8, #1                          ; =0x1
	b	LBB1_30
LBB1_35:                                ;   in Loop: Header=BB1_20 Depth=1
	cbz	w0, LBB1_38
; %bb.36:                               ;   in Loop: Header=BB1_20 Depth=1
	fcmp	d8, d12
	ldr	x2, [sp, #120]                  ; 8-byte Folded Reload
	ldr	x15, [sp, #96]                  ; 8-byte Folded Reload
	b.ge	LBB1_39
; %bb.37:                               ;   in Loop: Header=BB1_20 Depth=1
	mov	w14, #0                         ; =0x0
	b	LBB1_51
LBB1_38:                                ;   in Loop: Header=BB1_20 Depth=1
	sub	x8, x29, #224
	add	x8, x8, x20, lsl #2
	ldur	w8, [x8, #-4]
	asr	w14, w8, #23
	b	LBB1_25
LBB1_39:                                ;   in Loop: Header=BB1_20 Depth=1
	mov	w14, #2                         ; =0x2
	cmp	w2, #1
	b.ge	LBB1_27
LBB1_40:                                ;   in Loop: Header=BB1_20 Depth=1
	mov	w8, #1                          ; =0x1
LBB1_41:                                ;   in Loop: Header=BB1_20 Depth=1
	cmp	w0, #1
	b.lt	LBB1_47
; %bb.42:                               ;   in Loop: Header=BB1_20 Depth=1
	ldr	w9, [sp, #48]                   ; 4-byte Folded Reload
	cmp	w9, #25
	b.eq	LBB1_45
; %bb.43:                               ;   in Loop: Header=BB1_20 Depth=1
	cmp	w9, #26
	b.ne	LBB1_47
; %bb.44:                               ;   in Loop: Header=BB1_20 Depth=1
	mov	w9, #4194303                    ; =0x3fffff
	b	LBB1_46
LBB1_45:                                ;   in Loop: Header=BB1_20 Depth=1
	mov	w9, #8388607                    ; =0x7fffff
LBB1_46:                                ;   in Loop: Header=BB1_20 Depth=1
	sub	x10, x29, #224
	add	x10, x10, x20, lsl #2
	ldur	w11, [x10, #-4]
	and	w9, w11, w9
	stur	w9, [x10, #-4]
LBB1_47:                                ;   in Loop: Header=BB1_20 Depth=1
	add	w13, w13, #1
	cmp	w14, #2
	b.ne	LBB1_51
; %bb.48:                               ;   in Loop: Header=BB1_20 Depth=1
	fsub	d8, d11, d8
	tbnz	w8, #0, LBB1_50
; %bb.49:                               ;   in Loop: Header=BB1_20 Depth=1
	fmov	d0, #1.00000000
	str	w13, [sp, #44]                  ; 4-byte Folded Spill
	bl	_scalbn
	ldr	x15, [sp, #96]                  ; 8-byte Folded Reload
	ldr	x2, [sp, #120]                  ; 8-byte Folded Reload
	ldr	w0, [sp, #108]                  ; 4-byte Folded Reload
	ldr	w13, [sp, #44]                  ; 4-byte Folded Reload
	fsub	d8, d8, d0
LBB1_50:                                ;   in Loop: Header=BB1_20 Depth=1
	mov	w14, #2                         ; =0x2
LBB1_51:                                ;   in Loop: Header=BB1_20 Depth=1
	fcmp	d8, #0.0
	b.ne	LBB1_82
; %bb.52:                               ;   in Loop: Header=BB1_20 Depth=1
	ldr	x8, [sp, #112]                  ; 8-byte Folded Reload
	cmp	w2, w8
	b.le	LBB1_67
; %bb.53:                               ;   in Loop: Header=BB1_20 Depth=1
	sub	x8, x20, x15
	cmp	x8, #4
	b.hs	LBB1_55
; %bb.54:                               ;   in Loop: Header=BB1_20 Depth=1
	mov	w11, #0                         ; =0x0
	mov	x10, x20
	b	LBB1_65
LBB1_55:                                ;   in Loop: Header=BB1_20 Depth=1
	cmp	x8, #16
	b.hs	LBB1_57
; %bb.56:                               ;   in Loop: Header=BB1_20 Depth=1
	mov	x9, #0                          ; =0x0
	mov	w11, #0                         ; =0x0
	b	LBB1_61
LBB1_57:                                ;   in Loop: Header=BB1_20 Depth=1
	and	x9, x8, #0xfffffffffffffff0
	ldr	x10, [sp, #24]                  ; 8-byte Folded Reload
	add	x10, x10, x20, lsl #2
	movi.2d	v0, #0000000000000000
	mov	x11, x9
	movi.2d	v1, #0000000000000000
	movi.2d	v2, #0000000000000000
	movi.2d	v3, #0000000000000000
LBB1_58:                                ;   Parent Loop BB1_20 Depth=1
                                        ; =>  This Inner Loop Header: Depth=2
	ldp	q5, q4, [x10]
	rev64.4s	v4, v4
	ext.16b	v4, v4, v4, #8
	rev64.4s	v5, v5
	ext.16b	v5, v5, v5, #8
	ldp	q7, q6, [x10, #-32]
	rev64.4s	v6, v6
	ext.16b	v6, v6, v6, #8
	rev64.4s	v7, v7
	ext.16b	v7, v7, v7, #8
	orr.16b	v0, v4, v0
	orr.16b	v1, v5, v1
	orr.16b	v2, v6, v2
	orr.16b	v3, v7, v3
	sub	x10, x10, #64
	sub	x11, x11, #16
	cbnz	x11, LBB1_58
; %bb.59:                               ;   in Loop: Header=BB1_20 Depth=1
	orr.16b	v0, v1, v0
	orr.16b	v0, v2, v0
	orr.16b	v0, v3, v0
	ext.16b	v1, v0, v0, #8
	orr.8b	v0, v0, v1
	fmov	x10, d0
	lsr	x11, x10, #32
	orr	w11, w10, w11
	cmp	x8, x9
	b.eq	LBB1_66
; %bb.60:                               ;   in Loop: Header=BB1_20 Depth=1
	tst	x8, #0xc
	b.eq	LBB1_64
LBB1_61:                                ;   in Loop: Header=BB1_20 Depth=1
	and	x12, x8, #0xfffffffffffffffc
	sub	x10, x20, x12
	movi.2d	v0, #0000000000000000
	mov.s	v0[0], w11
	ldr	x11, [sp, #32]                  ; 8-byte Folded Reload
	sub	x11, x11, x9, lsl #2
	add	x11, x11, x20, lsl #2
	sub	x9, x9, x12
LBB1_62:                                ;   Parent Loop BB1_20 Depth=1
                                        ; =>  This Inner Loop Header: Depth=2
	ldr	q1, [x11], #-16
	rev64.4s	v1, v1
	ext.16b	v1, v1, v1, #8
	orr.16b	v0, v1, v0
	adds	x9, x9, #4
	b.ne	LBB1_62
; %bb.63:                               ;   in Loop: Header=BB1_20 Depth=1
	ext.16b	v1, v0, v0, #8
	orr.8b	v0, v0, v1
	fmov	x9, d0
	lsr	x11, x9, #32
	orr	w11, w9, w11
	cmp	x8, x12
	b.ne	LBB1_65
	b	LBB1_66
LBB1_64:                                ;   in Loop: Header=BB1_20 Depth=1
	sub	x10, x20, x9
LBB1_65:                                ;   Parent Loop BB1_20 Depth=1
                                        ; =>  This Inner Loop Header: Depth=2
	ldr	w8, [x25, x10, lsl #2]
	sub	x10, x10, #1
	orr	w11, w8, w11
	cmp	x10, x15
	b.gt	LBB1_65
LBB1_66:                                ;   in Loop: Header=BB1_20 Depth=1
	cbnz	w11, LBB1_85
LBB1_67:                                ;   in Loop: Header=BB1_20 Depth=1
	mov	w8, #1                          ; =0x1
	ldp	x15, x9, [sp, #80]              ; 16-byte Folded Reload
	ldr	x10, [sp, #64]                  ; 8-byte Folded Reload
LBB1_68:                                ;   Parent Loop BB1_20 Depth=1
                                        ; =>  This Inner Loop Header: Depth=2
	ldr	w11, [x10], #-4
	cbnz	w11, LBB1_71
; %bb.69:                               ;   in Loop: Header=BB1_68 Depth=2
	add	w8, w8, #1
	subs	x9, x9, #1
	b.ne	LBB1_68
; %bb.70:                               ;   in Loop: Header=BB1_20 Depth=1
	ldr	w8, [sp, #52]                   ; 4-byte Folded Reload
LBB1_71:                                ;   in Loop: Header=BB1_20 Depth=1
	add	w8, w8, w2
	sxtw	x9, w8
	add	w10, w22, w2
	ldr	x16, [sp, #72]                  ; 8-byte Folded Reload
	b	LBB1_74
LBB1_72:                                ;   in Loop: Header=BB1_74 Depth=2
	movi.2d	v0, #0000000000000000
LBB1_73:                                ;   in Loop: Header=BB1_74 Depth=2
	str	d0, [x26, x20, lsl #3]
	add	w10, w10, #1
	cmp	x20, x9
	b.ge	LBB1_19
LBB1_74:                                ;   Parent Loop BB1_20 Depth=1
                                        ; =>  This Loop Header: Depth=2
                                        ;       Child Loop BB1_78 Depth 3
                                        ;       Child Loop BB1_81 Depth 3
	mov	x11, x20
	add	x20, x20, #1
	ldr	s0, [x21, x20, lsl #2]
	add	w11, w22, w11
	sshll.2d	v0, v0, #0
	scvtf	d0, d0
	str	d0, [x27, w11, sxtw #3]
	cmp	w22, #1
	b.lt	LBB1_72
; %bb.75:                               ;   in Loop: Header=BB1_74 Depth=2
	sbfiz	x11, x10, #3, #32
	cmp	w22, #8
	b.hs	LBB1_77
; %bb.76:                               ;   in Loop: Header=BB1_74 Depth=2
	mov	x13, #0                         ; =0x0
	movi.2d	v0, #0000000000000000
	b	LBB1_80
LBB1_77:                                ;   in Loop: Header=BB1_74 Depth=2
	add	x12, x16, x11
	movi.2d	v0, #0000000000000000
	mov	x13, x15
	mov	x14, x28
LBB1_78:                                ;   Parent Loop BB1_20 Depth=1
                                        ;     Parent Loop BB1_74 Depth=2
                                        ; =>    This Inner Loop Header: Depth=3
	ldp	q1, q2, [x13, #-32]
	ldp	q3, q4, [x13], #64
	ldp	q6, q5, [x12]
	ext.16b	v5, v5, v5, #8
	ext.16b	v6, v6, v6, #8
	ldp	q16, q7, [x12, #-32]
	ext.16b	v7, v7, v7, #8
	ext.16b	v16, v16, v16, #8
	fmul.2d	v1, v1, v5
	mov	d5, v1[1]
	fmul.2d	v2, v2, v6
	mov	d6, v2[1]
	fmul.2d	v3, v3, v7
	mov	d7, v3[1]
	fmul.2d	v4, v4, v16
	mov	d16, v4[1]
	fadd	d0, d0, d1
	fadd	d0, d0, d5
	fadd	d0, d0, d2
	fadd	d0, d0, d6
	fadd	d0, d0, d3
	fadd	d0, d0, d7
	fadd	d0, d0, d4
	fadd	d0, d0, d16
	sub	x12, x12, #64
	sub	x14, x14, #8
	cbnz	x14, LBB1_78
; %bb.79:                               ;   in Loop: Header=BB1_74 Depth=2
	mov	x13, x28
	cmp	x28, x19
	b.eq	LBB1_73
LBB1_80:                                ;   in Loop: Header=BB1_74 Depth=2
	lsl	x12, x13, #3
	sub	x11, x11, x12
	add	x11, x27, x11
	add	x12, x23, x12
	sub	x13, x19, x13
LBB1_81:                                ;   Parent Loop BB1_20 Depth=1
                                        ;     Parent Loop BB1_74 Depth=2
                                        ; =>    This Inner Loop Header: Depth=3
	ldr	d1, [x12], #8
	ldr	d2, [x11], #-8
	fmadd	d0, d1, d2, d0
	subs	x13, x13, #1
	b.ne	LBB1_81
	b	LBB1_73
LBB1_82:
	mov	x23, x14
	mov	x22, x13
	mov	w8, #24                         ; =0x18
	ldr	w24, [sp, #48]                  ; 4-byte Folded Reload
	sub	w0, w8, w24
	mov.16b	v0, v8
	bl	_scalbn
	mov	x8, #4715268809856909312        ; =0x4170000000000000
	fmov	d1, x8
	fcmp	d0, d1
	ldr	x19, [sp, #16]                  ; 8-byte Folded Reload
	ldr	w21, [sp, #12]                  ; 4-byte Folded Reload
	b.ge	LBB1_84
; %bb.83:
	fcvtzs	w8, d0
	sub	x9, x29, #224
	str	w8, [x9, x20, lsl #2]
	ldr	w0, [sp, #108]                  ; 4-byte Folded Reload
	ldr	x2, [sp, #120]                  ; 8-byte Folded Reload
	b	LBB1_87
LBB1_84:
	mov	x8, #4499096027743125504        ; =0x3e70000000000000
	fmov	d1, x8
	fmul	d1, d0, d1
	fcvtzs	w8, d1
	scvtf	d1, w8
	mov	x9, #-4508103226997866496       ; =0xc170000000000000
	fmov	d2, x9
	fmadd	d0, d1, d2, d0
	fcvtzs	w9, d0
	sub	x10, x29, #224
	str	w9, [x10, x20, lsl #2]
	add	x9, x20, #1
	str	w8, [x10, x9, lsl #2]
	mov	x2, x9
	mov	x0, x24
	b	LBB1_87
LBB1_85:
	mov	x23, x14
	mov	x22, x13
	sub	x8, x29, #224
	add	x8, x8, w2, sxtw #2
	sub	x8, x8, #4
	ldr	x19, [sp, #16]                  ; 8-byte Folded Reload
	ldr	w21, [sp, #12]                  ; 4-byte Folded Reload
LBB1_86:                                ; =>This Inner Loop Header: Depth=1
	sub	w0, w0, #24
	ldr	w9, [x8], #-4
	sub	w2, w2, #1
	cbz	w9, LBB1_86
LBB1_87:
	tbnz	w2, #31, LBB1_101
; %bb.88:
	fmov	d0, #1.00000000
	mov	x20, x2
	bl	_scalbn
	mov	x2, x20
	mov	w8, w2
	sub	x9, x29, #224
	add	x10, sp, #128
	mov	x11, #4499096027743125504       ; =0x3e70000000000000
	fmov	d1, x11
	mov	x11, x8
LBB1_89:                                ; =>This Inner Loop Header: Depth=1
	ldr	s2, [x9, x11, lsl #2]
	sshll.2d	v2, v2, #0
	scvtf	d2, d2
	fmul	d2, d0, d2
	fmul	d0, d0, d1
	str	d2, [x10, x11, lsl #3]
	sub	x11, x11, #1
	cmn	x11, #1
	b.ne	LBB1_89
; %bb.90:
	add	x9, sp, #128
	add	x10, x9, x8, lsl #3
	add	x10, x10, #32
	add	x11, sp, #288
Lloh15:
	adrp	x12, __ZZ17__kernel_rem_pio2PdS_iiiPKiE4PIo2@PAGE
Lloh16:
	add	x12, x12, __ZZ17__kernel_rem_pio2PdS_iiiPKiE4PIo2@PAGEOFF
Lloh17:
	adrp	x13, __ZZ17__kernel_rem_pio2PdS_iiiPKiE4PIo2@PAGE+32
Lloh18:
	add	x13, x13, __ZZ17__kernel_rem_pio2PdS_iiiPKiE4PIo2@PAGEOFF+32
	b	LBB1_93
LBB1_91:                                ;   in Loop: Header=BB1_93 Depth=1
	movi.2d	v0, #0000000000000000
LBB1_92:                                ;   in Loop: Header=BB1_93 Depth=1
	str	d0, [x11, w14, sxtw #3]
	sub	x10, x10, #8
	cmp	x8, #0
	sub	x8, x8, #1
	b.le	LBB1_101
LBB1_93:                                ; =>This Loop Header: Depth=1
                                        ;     Child Loop BB1_97 Depth 2
                                        ;     Child Loop BB1_100 Depth 2
	sub	w14, w2, w8
	ldr	x15, [sp, #112]                 ; 8-byte Folded Reload
	cmp	w15, w14
	csel	w16, w15, w14, lt
	tbnz	w16, #31, LBB1_91
; %bb.94:                               ;   in Loop: Header=BB1_93 Depth=1
	add	w15, w16, #1
	cmp	w16, #7
	b.hs	LBB1_96
; %bb.95:                               ;   in Loop: Header=BB1_93 Depth=1
	mov	x16, #0                         ; =0x0
	movi.2d	v0, #0000000000000000
	b	LBB1_99
LBB1_96:                                ;   in Loop: Header=BB1_93 Depth=1
	and	x16, x15, #0xfffffff8
	movi.2d	v0, #0000000000000000
	mov	x17, x10
	mov	x0, x13
	mov	x1, x16
LBB1_97:                                ;   Parent Loop BB1_93 Depth=1
                                        ; =>  This Inner Loop Header: Depth=2
	ldp	q1, q2, [x0, #-32]
	ldp	q3, q4, [x0], #64
	ldp	q5, q6, [x17, #-32]
	ldp	q7, q16, [x17], #64
	fmul.2d	v1, v1, v5
	mov	d5, v1[1]
	fmul.2d	v2, v2, v6
	mov	d6, v2[1]
	fmul.2d	v3, v3, v7
	mov	d7, v3[1]
	fmul.2d	v4, v4, v16
	mov	d16, v4[1]
	fadd	d0, d0, d1
	fadd	d0, d0, d5
	fadd	d0, d0, d2
	fadd	d0, d0, d6
	fadd	d0, d0, d3
	fadd	d0, d0, d7
	fadd	d0, d0, d4
	fadd	d0, d0, d16
	subs	x1, x1, #8
	b.ne	LBB1_97
; %bb.98:                               ;   in Loop: Header=BB1_93 Depth=1
	cmp	x16, x15
	b.eq	LBB1_92
LBB1_99:                                ;   in Loop: Header=BB1_93 Depth=1
	add	x17, x16, x8
	add	x17, x9, x17, lsl #3
	add	x0, x12, x16, lsl #3
	sub	x15, x15, x16
LBB1_100:                               ;   Parent Loop BB1_93 Depth=1
                                        ; =>  This Inner Loop Header: Depth=2
	ldr	d1, [x0], #8
	ldr	d2, [x17], #8
	fmadd	d0, d1, d2, d0
	subs	x15, x15, #1
	b.ne	LBB1_100
	b	LBB1_92
LBB1_101:
	sub	w8, w21, #1
	mov	x13, x22
	cmp	w8, #2
	b.lo	LBB1_112
; %bb.102:
	cbz	w21, LBB1_115
; %bb.103:
	cmp	w21, #3
	b.ne	LBB1_149
; %bb.104:
	movi.2d	v0, #0000000000000000
	cmp	w2, #1
	b.lt	LBB1_145
; %bb.105:
	mov	w8, w2
	add	x9, sp, #288
	add	x9, x9, w2, uxtw #3
	ldr	d1, [x9], #-8
	add	x10, x8, #1
LBB1_106:                               ; =>This Inner Loop Header: Depth=1
	ldr	d2, [x9]
	fadd	d3, d2, d1
	fsub	d2, d2, d3
	fadd	d1, d1, d2
	stp	d3, d1, [x9], #-8
	sub	x10, x10, #1
	mov.16b	v1, v3
	cmp	x10, #1
	b.hi	LBB1_106
; %bb.107:
	cmp	w2, #1
	b.eq	LBB1_145
; %bb.108:
	add	x9, sp, #288
	add	x9, x9, x8, lsl #3
	ldr	d0, [x9], #-8
	add	x10, x8, #1
LBB1_109:                               ; =>This Inner Loop Header: Depth=1
	ldr	d1, [x9]
	fadd	d2, d1, d0
	fsub	d1, d1, d2
	fadd	d0, d0, d1
	stp	d2, d0, [x9], #-8
	sub	x10, x10, #1
	mov.16b	v0, v2
	cmp	x10, #2
	b.hi	LBB1_109
; %bb.110:
	subs	x9, x8, #2
	csel	x9, xzr, x9, lo
	movi.2d	v0, #0000000000000000
	cmp	x9, #3
	b.hs	LBB1_140
; %bb.111:
	mov	x9, x8
	b	LBB1_143
LBB1_112:
	tbnz	w2, #31, LBB1_118
; %bb.113:
	mov	w11, w2
	movi.2d	v0, #0000000000000000
	cmp	w2, #3
	b.hs	LBB1_119
; %bb.114:
	mov	x8, x11
	b	LBB1_122
LBB1_115:
	tbnz	w2, #31, LBB1_133
; %bb.116:
	mov	w11, w2
	movi.2d	v0, #0000000000000000
	cmp	w2, #3
	b.hs	LBB1_134
; %bb.117:
	mov	x8, x11
	b	LBB1_137
LBB1_118:
	movi.2d	v0, #0000000000000000
	b	LBB1_124
LBB1_119:
	add	x9, x11, #1
	and	x10, x9, #0xfffffffc
	sub	x8, x11, x10
	add	x12, sp, #288
	add	x11, x12, x11, lsl #3
	sub	x11, x11, #8
	mov	x12, x10
LBB1_120:                               ; =>This Inner Loop Header: Depth=1
	ldp	d2, d1, [x11]
	ldp	d4, d3, [x11, #-16]
	fadd	d0, d0, d1
	fadd	d0, d0, d2
	fadd	d0, d0, d3
	fadd	d0, d0, d4
	sub	x11, x11, #32
	sub	x12, x12, #4
	cbnz	x12, LBB1_120
; %bb.121:
	cmp	x9, x10
	b.eq	LBB1_124
LBB1_122:
	add	x9, sp, #288
LBB1_123:                               ; =>This Inner Loop Header: Depth=1
	ldr	d1, [x9, x8, lsl #3]
	fadd	d0, d0, d1
	sub	x8, x8, #1
	cmn	x8, #1
	b.ne	LBB1_123
LBB1_124:
	fneg	d1, d0
	cmp	w23, #0
	fcsel	d1, d0, d1, eq
	str	d1, [x19]
	ldr	d1, [sp, #288]
	fsub	d0, d1, d0
	cmp	w2, #1
	b.lt	LBB1_132
; %bb.125:
	cmp	w2, #8
	b.hs	LBB1_127
; %bb.126:
	mov	w8, #1                          ; =0x1
	b	LBB1_130
LBB1_127:
	mov	w9, w2
	and	x10, x9, #0x7ffffff8
	orr	x8, x10, #0x1
	add	x11, sp, #288
	add	x11, x11, #40
	mov	x12, x10
LBB1_128:                               ; =>This Inner Loop Header: Depth=1
	ldp	q1, q2, [x11, #-32]
	mov	d3, v1[1]
	mov	d4, v2[1]
	ldp	q5, q6, [x11], #64
	mov	d7, v5[1]
	mov	d16, v6[1]
	fadd	d0, d0, d1
	fadd	d0, d0, d3
	fadd	d0, d0, d2
	fadd	d0, d0, d4
	fadd	d0, d0, d5
	fadd	d0, d0, d7
	fadd	d0, d0, d6
	fadd	d0, d0, d16
	subs	x12, x12, #8
	b.ne	LBB1_128
; %bb.129:
	cmp	x10, x9
	b.eq	LBB1_132
LBB1_130:
	add	w10, w2, #1
	add	x9, sp, #288
	add	x9, x9, x8, lsl #3
	sub	x8, x10, x8
LBB1_131:                               ; =>This Inner Loop Header: Depth=1
	ldr	d1, [x9], #8
	fadd	d0, d0, d1
	subs	x8, x8, #1
	b.ne	LBB1_131
LBB1_132:
	fneg	d1, d0
	cmp	w23, #0
	fcsel	d0, d0, d1, eq
	str	d0, [x19, #8]
	b	LBB1_149
LBB1_133:
	movi.2d	v0, #0000000000000000
	b	LBB1_139
LBB1_134:
	add	x9, x11, #1
	and	x10, x9, #0xfffffffc
	sub	x8, x11, x10
	add	x12, sp, #288
	add	x11, x12, x11, lsl #3
	sub	x11, x11, #8
	mov	x12, x10
LBB1_135:                               ; =>This Inner Loop Header: Depth=1
	ldp	d2, d1, [x11]
	ldp	d4, d3, [x11, #-16]
	fadd	d0, d0, d1
	fadd	d0, d0, d2
	fadd	d0, d0, d3
	fadd	d0, d0, d4
	sub	x11, x11, #32
	sub	x12, x12, #4
	cbnz	x12, LBB1_135
; %bb.136:
	cmp	x9, x10
	b.eq	LBB1_139
LBB1_137:
	add	x9, sp, #288
LBB1_138:                               ; =>This Inner Loop Header: Depth=1
	ldr	d1, [x9, x8, lsl #3]
	fadd	d0, d0, d1
	sub	x8, x8, #1
	cmn	x8, #1
	b.ne	LBB1_138
LBB1_139:
	fneg	d1, d0
	cmp	w23, #0
	fcsel	d0, d0, d1, eq
	str	d0, [x19]
	b	LBB1_149
LBB1_140:
	add	x10, x9, #1
	and	x11, x10, #0xfffffffc
	sub	x9, x8, x11
	add	x12, sp, #288
	add	x8, x12, x8, lsl #3
	sub	x8, x8, #8
	mov	x12, x11
LBB1_141:                               ; =>This Inner Loop Header: Depth=1
	ldp	d2, d1, [x8]
	ldp	d4, d3, [x8, #-16]
	fadd	d0, d0, d1
	fadd	d0, d0, d2
	fadd	d0, d0, d3
	fadd	d0, d0, d4
	sub	x8, x8, #32
	sub	x12, x12, #4
	cbnz	x12, LBB1_141
; %bb.142:
	cmp	x10, x11
	b.eq	LBB1_145
LBB1_143:
	add	x8, x9, #1
	add	x10, sp, #288
	add	x9, x10, x9, lsl #3
LBB1_144:                               ; =>This Inner Loop Header: Depth=1
	ldr	d1, [x9], #-8
	fadd	d0, d0, d1
	sub	x8, x8, #1
	cmp	x8, #2
	b.hi	LBB1_144
LBB1_145:
	ldr	d1, [sp, #288]
	cbz	w23, LBB1_147
; %bb.146:
	fneg	d1, d1
	ldr	d2, [sp, #296]
	fneg	d2, d2
	stp	d1, d2, [x19]
	fneg	d0, d0
	b	LBB1_148
LBB1_147:
	ldr	d2, [sp, #296]
	stp	d1, d2, [x19]
LBB1_148:
	str	d0, [x19, #16]
LBB1_149:
	ldur	x8, [x29, #-144]
Lloh19:
	adrp	x9, ___stack_chk_guard@GOTPAGE
Lloh20:
	ldr	x9, [x9, ___stack_chk_guard@GOTPAGEOFF]
Lloh21:
	ldr	x9, [x9]
	cmp	x9, x8
	b.ne	LBB1_151
; %bb.150:
	and	w0, w13, #0x7
	add	sp, sp, #704
	ldp	x29, x30, [sp, #128]            ; 16-byte Folded Reload
	ldp	x20, x19, [sp, #112]            ; 16-byte Folded Reload
	ldp	x22, x21, [sp, #96]             ; 16-byte Folded Reload
	ldp	x24, x23, [sp, #80]             ; 16-byte Folded Reload
	ldp	x26, x25, [sp, #64]             ; 16-byte Folded Reload
	ldp	x28, x27, [sp, #48]             ; 16-byte Folded Reload
	ldp	d9, d8, [sp, #32]               ; 16-byte Folded Reload
	ldp	d11, d10, [sp, #16]             ; 16-byte Folded Reload
	ldp	d13, d12, [sp], #144            ; 16-byte Folded Reload
	ret
LBB1_151:
	bl	___stack_chk_fail
	.loh AdrpAdd	Lloh13, Lloh14
	.loh AdrpLdrGotLdr	Lloh10, Lloh11, Lloh12
	.loh AdrpAdd	Lloh17, Lloh18
	.loh AdrpAdd	Lloh15, Lloh16
	.loh AdrpLdrGotLdr	Lloh19, Lloh20, Lloh21
	.cfi_endproc
                                        ; -- End function
	.globl	__Z12__kernel_sinddi            ; -- Begin function _Z12__kernel_sinddi
	.p2align	2
__Z12__kernel_sinddi:                   ; @_Z12__kernel_sinddi
	.cfi_startproc
; %bb.0:
	fmov	x8, d0
	fcvtzs	w9, d0
	ubfx	x8, x8, #54, #9
	cmp	x8, #248
	ccmp	w9, #0, #0, ls
	b.eq	LBB2_3
; %bb.1:
	fmul	d3, d0, d0
	fmul	d2, d0, d3
	mov	x8, #40171                      ; =0x9ceb
	movk	x8, #35371, lsl #16
	movk	x8, #58854, lsl #32
	movk	x8, #48730, lsl #48
	fmov	d4, x8
	mov	x8, #54652                      ; =0xd57c
	movk	x8, #23247, lsl #16
	movk	x8, #55610, lsl #32
	movk	x8, #15845, lsl #48
	fmov	d5, x8
	fmadd	d4, d3, d5, d4
	mov	x8, #65149                      ; =0xfe7d
	movk	x8, #22449, lsl #16
	movk	x8, #7651, lsl #32
	movk	x8, #16071, lsl #48
	fmov	d5, x8
	fmadd	d4, d3, d4, d5
	mov	x8, #25045                      ; =0x61d5
	movk	x8, #6593, lsl #16
	movk	x8, #416, lsl #32
	movk	x8, #48938, lsl #48
	fmov	d5, x8
	fmadd	d4, d3, d4, d5
	mov	x8, #63654                      ; =0xf8a6
	movk	x8, #4368, lsl #16
	movk	x8, #4369, lsl #32
	movk	x8, #16257, lsl #48
	fmov	d5, x8
	fmadd	d4, d3, d4, d5
	cbz	w0, LBB2_4
; %bb.2:
	fmul	d4, d2, d4
	fmov	d5, #0.50000000
	fnmsub	d4, d1, d5, d4
	fnmsub	d1, d3, d4, d1
	mov	x8, #6148914691236517205        ; =0x5555555555555555
	movk	x8, #21833
	movk	x8, #16325, lsl #48
	fmov	d3, x8
	fmadd	d1, d2, d3, d1
	fsub	d0, d0, d1
LBB2_3:
	ret
LBB2_4:
	mov	x8, #6148914691236517205        ; =0x5555555555555555
	movk	x8, #21833
	movk	x8, #49093, lsl #48
	fmov	d1, x8
	fmadd	d1, d3, d4, d1
	fmadd	d0, d2, d1, d0
	ret
	.cfi_endproc
                                        ; -- End function
	.globl	__Z12__kernel_cosdd             ; -- Begin function _Z12__kernel_cosdd
	.p2align	2
__Z12__kernel_cosdd:                    ; @_Z12__kernel_cosdd
	.cfi_startproc
; %bb.0:
	fmov	x8, d0
	lsr	x9, x8, #32
	fcvtzs	w10, d0
	fmov	d2, #1.00000000
	ubfx	w9, w9, #22, #9
	cmp	w9, #248
	ccmp	w10, #0, #0, ls
	b.eq	LBB3_4
; %bb.1:
	ubfx	x8, x8, #32, #31
	fmul	d2, d0, d0
	mov	x9, #45508                      ; =0xb1c4
	movk	x9, #48564, lsl #16
	movk	x9, #61086, lsl #32
	movk	x9, #15905, lsl #48
	fmov	d3, x9
	mov	x9, #14548                      ; =0x38d4
	movk	x9, #48776, lsl #16
	movk	x9, #64233, lsl #32
	movk	x9, #48552, lsl #48
	fmov	d4, x9
	fmadd	d3, d2, d4, d3
	mov	x9, #21165                      ; =0x52ad
	movk	x9, #32924, lsl #16
	movk	x9, #32335, lsl #32
	movk	x9, #48786, lsl #48
	fmov	d4, x9
	fmadd	d3, d2, d3, d4
	mov	x9, #5520                       ; =0x1590
	movk	x9, #6603, lsl #16
	movk	x9, #416, lsl #32
	movk	x9, #16122, lsl #48
	fmov	d4, x9
	fmadd	d3, d2, d3, d4
	mov	x9, #20855                      ; =0x5177
	movk	x9, #5825, lsl #16
	movk	x9, #49516, lsl #32
	movk	x9, #48982, lsl #48
	fmov	d4, x9
	fmadd	d3, d2, d3, d4
	mov	x9, #6148914691236517205        ; =0x5555555555555555
	movk	x9, #21836
	movk	x9, #16293, lsl #48
	fmov	d4, x9
	fmadd	d3, d2, d3, d4
	fmul	d3, d2, d3
	mov	w9, #13106                      ; =0x3332
	movk	w9, #16339, lsl #16
	cmp	w8, w9
	b.hi	LBB3_3
; %bb.2:
	fmul	d0, d0, d1
	fnmsub	d0, d2, d3, d0
	fmov	d1, #-0.50000000
	fmadd	d0, d2, d1, d0
	fmov	d1, #1.00000000
	fadd	d0, d0, d1
	ret
LBB3_3:
	sub	w9, w8, #512, lsl #12           ; =2097152
	lsl	x9, x9, #32
	fmov	d4, x9
	mov	w9, #1072234496                 ; =0x3fe90000
	cmp	w8, w9
	fmov	d5, #0.28125000
	fcsel	d4, d5, d4, hi
	fmov	d5, #0.50000000
	fnmsub	d5, d2, d5, d4
	fmov	d6, #1.00000000
	fsub	d4, d6, d4
	fmul	d0, d0, d1
	fnmsub	d0, d2, d3, d0
	fsub	d0, d0, d5
	fadd	d2, d4, d0
LBB3_4:
	mov.16b	v0, v2
	ret
	.cfi_endproc
                                        ; -- End function
	.globl	__Z4sined                       ; -- Begin function _Z4sined
	.p2align	2
__Z4sined:                              ; @_Z4sined
	.cfi_startproc
; %bb.0:
	sub	sp, sp, #48
	stp	x29, x30, [sp, #32]             ; 16-byte Folded Spill
	add	x29, sp, #32
	.cfi_def_cfa w29, 16
	.cfi_offset w30, -8
	.cfi_offset w29, -16
Lloh22:
	adrp	x8, ___stack_chk_guard@GOTPAGE
Lloh23:
	ldr	x8, [x8, ___stack_chk_guard@GOTPAGEOFF]
Lloh24:
	ldr	x8, [x8]
	stur	x8, [x29, #-8]
	add	x0, sp, #8
	bl	__Z18__ieee754_rem_pio2dPd
	ldr	d2, [sp, #16]
	cbnz	w0, LBB4_4
; %bb.1:
	fcmp	d2, #0.0
	b.ne	LBB4_4
; %bb.2:
	ldr	d0, [sp, #8]
	fmov	x8, d0
	fcvtzs	w9, d0
	ubfx	x8, x8, #54, #9
	cmp	x8, #248
	ccmp	w9, #0, #0, ls
	b.eq	LBB4_22
; %bb.3:
	fmul	d1, d0, d0
	fmul	d2, d0, d1
	mov	x8, #40171                      ; =0x9ceb
	movk	x8, #35371, lsl #16
	movk	x8, #58854, lsl #32
	movk	x8, #48730, lsl #48
	fmov	d3, x8
	mov	x8, #54652                      ; =0xd57c
	movk	x8, #23247, lsl #16
	movk	x8, #55610, lsl #32
	movk	x8, #15845, lsl #48
	fmov	d4, x8
	fmadd	d3, d1, d4, d3
	mov	x8, #65149                      ; =0xfe7d
	movk	x8, #22449, lsl #16
	movk	x8, #7651, lsl #32
	movk	x8, #16071, lsl #48
	fmov	d4, x8
	fmadd	d3, d1, d3, d4
	mov	x8, #25045                      ; =0x61d5
	movk	x8, #6593, lsl #16
	movk	x8, #416, lsl #32
	movk	x8, #48938, lsl #48
	fmov	d4, x8
	fmadd	d3, d1, d3, d4
	mov	x8, #63654                      ; =0xf8a6
	movk	x8, #4368, lsl #16
	movk	x8, #4369, lsl #32
	movk	x8, #16257, lsl #48
	fmov	d4, x8
	fmadd	d3, d1, d3, d4
	mov	x8, #6148914691236517205        ; =0x5555555555555555
	movk	x8, #21833
	movk	x8, #49093, lsl #48
	fmov	d4, x8
	fmadd	d1, d1, d3, d4
	fmadd	d0, d2, d1, d0
	b	LBB4_22
LBB4_4:
	and	w9, w0, #0x3
	ldr	d1, [sp, #8]
	fmov	x8, d1
	cmp	w9, #1
	b.gt	LBB4_8
; %bb.5:
	cbnz	w9, LBB4_12
; %bb.6:
	fcvtzs	w9, d1
	ubfx	x8, x8, #54, #9
	cmp	x8, #248
	ccmp	w9, #0, #0, ls
	b.eq	LBB4_18
; %bb.7:
	fmul	d0, d1, d1
	fmul	d3, d1, d0
	mov	x8, #40171                      ; =0x9ceb
	movk	x8, #35371, lsl #16
	movk	x8, #58854, lsl #32
	movk	x8, #48730, lsl #48
	fmov	d4, x8
	mov	x8, #54652                      ; =0xd57c
	movk	x8, #23247, lsl #16
	movk	x8, #55610, lsl #32
	movk	x8, #15845, lsl #48
	fmov	d5, x8
	fmadd	d4, d0, d5, d4
	mov	x8, #65149                      ; =0xfe7d
	movk	x8, #22449, lsl #16
	movk	x8, #7651, lsl #32
	movk	x8, #16071, lsl #48
	fmov	d5, x8
	fmadd	d4, d0, d4, d5
	mov	x8, #25045                      ; =0x61d5
	movk	x8, #6593, lsl #16
	movk	x8, #416, lsl #32
	movk	x8, #48938, lsl #48
	fmov	d5, x8
	fmadd	d4, d0, d4, d5
	mov	x8, #63654                      ; =0xf8a6
	movk	x8, #4368, lsl #16
	movk	x8, #4369, lsl #32
	movk	x8, #16257, lsl #48
	fmov	d5, x8
	fmadd	d4, d0, d4, d5
	fmul	d4, d3, d4
	fmov	d5, #0.50000000
	fnmsub	d4, d2, d5, d4
	fnmsub	d0, d0, d4, d2
	mov	x8, #6148914691236517205        ; =0x5555555555555555
	movk	x8, #21833
	movk	x8, #16325, lsl #48
	fmov	d2, x8
	fmadd	d0, d3, d2, d0
	fsub	d0, d1, d0
	b	LBB4_22
LBB4_8:
	cmp	w9, #2
	b.ne	LBB4_15
; %bb.9:
	fcvtzs	w9, d1
	ubfx	x8, x8, #54, #9
	cmp	x8, #248
	ccmp	w9, #0, #0, ls
	b.eq	LBB4_11
; %bb.10:
	fmul	d0, d1, d1
	fmul	d3, d1, d0
	mov	x8, #40171                      ; =0x9ceb
	movk	x8, #35371, lsl #16
	movk	x8, #58854, lsl #32
	movk	x8, #48730, lsl #48
	fmov	d4, x8
	mov	x8, #54652                      ; =0xd57c
	movk	x8, #23247, lsl #16
	movk	x8, #55610, lsl #32
	movk	x8, #15845, lsl #48
	fmov	d5, x8
	fmadd	d4, d0, d5, d4
	mov	x8, #65149                      ; =0xfe7d
	movk	x8, #22449, lsl #16
	movk	x8, #7651, lsl #32
	movk	x8, #16071, lsl #48
	fmov	d5, x8
	fmadd	d4, d0, d4, d5
	mov	x8, #25045                      ; =0x61d5
	movk	x8, #6593, lsl #16
	movk	x8, #416, lsl #32
	movk	x8, #48938, lsl #48
	fmov	d5, x8
	fmadd	d4, d0, d4, d5
	mov	x8, #63654                      ; =0xf8a6
	movk	x8, #4368, lsl #16
	movk	x8, #4369, lsl #32
	movk	x8, #16257, lsl #48
	fmov	d5, x8
	fmadd	d4, d0, d4, d5
	fmul	d4, d3, d4
	fmov	d5, #0.50000000
	fnmsub	d4, d2, d5, d4
	fnmsub	d0, d0, d4, d2
	mov	x8, #6148914691236517205        ; =0x5555555555555555
	movk	x8, #21833
	movk	x8, #16325, lsl #48
	fmov	d2, x8
	fmadd	d0, d3, d2, d0
	fsub	d1, d1, d0
LBB4_11:
	fneg	d0, d1
	b	LBB4_22
LBB4_12:
	lsr	x9, x8, #32
	fcvtzs	w10, d1
	fmov	d0, #1.00000000
	ubfx	w9, w9, #22, #9
	cmp	w9, #248
	ccmp	w10, #0, #0, ls
	b.eq	LBB4_22
; %bb.13:
	ubfx	x8, x8, #32, #31
	fmul	d0, d1, d1
	mov	x9, #45508                      ; =0xb1c4
	movk	x9, #48564, lsl #16
	movk	x9, #61086, lsl #32
	movk	x9, #15905, lsl #48
	fmov	d3, x9
	mov	x9, #14548                      ; =0x38d4
	movk	x9, #48776, lsl #16
	movk	x9, #64233, lsl #32
	movk	x9, #48552, lsl #48
	fmov	d4, x9
	fmadd	d3, d0, d4, d3
	mov	x9, #21165                      ; =0x52ad
	movk	x9, #32924, lsl #16
	movk	x9, #32335, lsl #32
	movk	x9, #48786, lsl #48
	fmov	d4, x9
	fmadd	d3, d0, d3, d4
	mov	x9, #5520                       ; =0x1590
	movk	x9, #6603, lsl #16
	movk	x9, #416, lsl #32
	movk	x9, #16122, lsl #48
	fmov	d4, x9
	fmadd	d3, d0, d3, d4
	mov	x9, #20855                      ; =0x5177
	movk	x9, #5825, lsl #16
	movk	x9, #49516, lsl #32
	movk	x9, #48982, lsl #48
	fmov	d4, x9
	fmadd	d3, d0, d3, d4
	mov	x9, #6148914691236517205        ; =0x5555555555555555
	movk	x9, #21836
	movk	x9, #16293, lsl #48
	fmov	d4, x9
	fmadd	d3, d0, d3, d4
	fmul	d3, d0, d3
	mov	w9, #13107                      ; =0x3333
	movk	w9, #16339, lsl #16
	cmp	w8, w9
	b.hs	LBB4_19
; %bb.14:
	fmul	d1, d2, d1
	fnmsub	d1, d0, d3, d1
	fmov	d2, #-0.50000000
	fmadd	d0, d0, d2, d1
	fmov	d1, #1.00000000
	fadd	d0, d0, d1
	b	LBB4_22
LBB4_15:
	lsr	x9, x8, #32
	fcvtzs	w10, d1
	fmov	d0, #1.00000000
	ubfx	w9, w9, #22, #9
	cmp	w9, #248
	ccmp	w10, #0, #0, ls
	b.eq	LBB4_21
; %bb.16:
	ubfx	x8, x8, #32, #31
	fmul	d0, d1, d1
	mov	x9, #45508                      ; =0xb1c4
	movk	x9, #48564, lsl #16
	movk	x9, #61086, lsl #32
	movk	x9, #15905, lsl #48
	fmov	d3, x9
	mov	x9, #14548                      ; =0x38d4
	movk	x9, #48776, lsl #16
	movk	x9, #64233, lsl #32
	movk	x9, #48552, lsl #48
	fmov	d4, x9
	fmadd	d3, d0, d4, d3
	mov	x9, #21165                      ; =0x52ad
	movk	x9, #32924, lsl #16
	movk	x9, #32335, lsl #32
	movk	x9, #48786, lsl #48
	fmov	d4, x9
	fmadd	d3, d0, d3, d4
	mov	x9, #5520                       ; =0x1590
	movk	x9, #6603, lsl #16
	movk	x9, #416, lsl #32
	movk	x9, #16122, lsl #48
	fmov	d4, x9
	fmadd	d3, d0, d3, d4
	mov	x9, #20855                      ; =0x5177
	movk	x9, #5825, lsl #16
	movk	x9, #49516, lsl #32
	movk	x9, #48982, lsl #48
	fmov	d4, x9
	fmadd	d3, d0, d3, d4
	mov	x9, #6148914691236517205        ; =0x5555555555555555
	movk	x9, #21836
	movk	x9, #16293, lsl #48
	fmov	d4, x9
	fmadd	d3, d0, d3, d4
	fmul	d3, d0, d3
	mov	w9, #13107                      ; =0x3333
	movk	w9, #16339, lsl #16
	cmp	w8, w9
	b.hs	LBB4_20
; %bb.17:
	fmul	d1, d2, d1
	fnmsub	d1, d0, d3, d1
	fmov	d2, #-0.50000000
	fmadd	d0, d0, d2, d1
	fmov	d1, #1.00000000
	fadd	d0, d0, d1
	b	LBB4_21
LBB4_18:
	mov.16b	v0, v1
	b	LBB4_22
LBB4_19:
	sub	w9, w8, #512, lsl #12           ; =2097152
	lsl	x9, x9, #32
	fmov	d4, x9
	mov	w9, #1072234496                 ; =0x3fe90000
	cmp	w8, w9
	fmov	d5, #0.28125000
	fcsel	d4, d5, d4, hi
	fmov	d5, #0.50000000
	fnmsub	d5, d0, d5, d4
	fmov	d6, #1.00000000
	fsub	d4, d6, d4
	fmul	d1, d2, d1
	fnmsub	d0, d0, d3, d1
	fsub	d0, d0, d5
	fadd	d0, d4, d0
	b	LBB4_22
LBB4_20:
	sub	w9, w8, #512, lsl #12           ; =2097152
	lsl	x9, x9, #32
	fmov	d4, x9
	mov	w9, #1072234496                 ; =0x3fe90000
	cmp	w8, w9
	fmov	d5, #0.28125000
	fcsel	d4, d5, d4, hi
	fmov	d5, #0.50000000
	fnmsub	d5, d0, d5, d4
	fmov	d6, #1.00000000
	fsub	d4, d6, d4
	fmul	d1, d2, d1
	fnmsub	d0, d0, d3, d1
	fsub	d0, d0, d5
	fadd	d0, d4, d0
LBB4_21:
	fneg	d0, d0
LBB4_22:
	ldur	x8, [x29, #-8]
Lloh25:
	adrp	x9, ___stack_chk_guard@GOTPAGE
Lloh26:
	ldr	x9, [x9, ___stack_chk_guard@GOTPAGEOFF]
Lloh27:
	ldr	x9, [x9]
	cmp	x9, x8
	b.ne	LBB4_24
; %bb.23:
	ldp	x29, x30, [sp, #32]             ; 16-byte Folded Reload
	add	sp, sp, #48
	ret
LBB4_24:
	bl	___stack_chk_fail
	.loh AdrpLdrGotLdr	Lloh22, Lloh23, Lloh24
	.loh AdrpLdrGotLdr	Lloh25, Lloh26, Lloh27
	.cfi_endproc
                                        ; -- End function
	.globl	_main                           ; -- Begin function main
	.p2align	2
_main:                                  ; @main
	.cfi_startproc
; %bb.0:
	cmp	w0, #2
	b.lt	LBB5_4
; %bb.1:
	sub	sp, sp, #96
	stp	d9, d8, [sp, #32]               ; 16-byte Folded Spill
	stp	x22, x21, [sp, #48]             ; 16-byte Folded Spill
	stp	x20, x19, [sp, #64]             ; 16-byte Folded Spill
	stp	x29, x30, [sp, #80]             ; 16-byte Folded Spill
	add	x29, sp, #80
	.cfi_def_cfa w29, 16
	.cfi_offset w30, -8
	.cfi_offset w29, -16
	.cfi_offset w19, -24
	.cfi_offset w20, -32
	.cfi_offset w21, -40
	.cfi_offset w22, -48
	.cfi_offset b8, -56
	.cfi_offset b9, -64
	mov	w8, w0
	add	x20, x1, #8
	sub	x21, x8, #1
Lloh28:
	adrp	x19, l_.str@PAGE
Lloh29:
	add	x19, x19, l_.str@PAGEOFF
LBB5_2:                                 ; =>This Inner Loop Header: Depth=1
	ldr	x0, [x20], #8
	bl	_atof
	mov.16b	v8, v0
	bl	__Z4sined
	mov.16b	v9, v0
	mov.16b	v0, v8
	bl	_sin
	stp	d9, d0, [sp, #8]
	str	d8, [sp]
	mov	x0, x19
	bl	_printf
	subs	x21, x21, #1
	b.ne	LBB5_2
; %bb.3:
	ldp	x29, x30, [sp, #80]             ; 16-byte Folded Reload
	ldp	x20, x19, [sp, #64]             ; 16-byte Folded Reload
	ldp	x22, x21, [sp, #48]             ; 16-byte Folded Reload
	ldp	d9, d8, [sp, #32]               ; 16-byte Folded Reload
	add	sp, sp, #96
LBB5_4:
	mov	w0, #0                          ; =0x0
	ret
	.loh AdrpAdd	Lloh28, Lloh29
	.cfi_endproc
                                        ; -- End function
	.section	__TEXT,__const
	.p2align	2, 0x0                          ; @_ZZ18__ieee754_rem_pio2dPdE11two_over_pi
__ZZ18__ieee754_rem_pio2dPdE11two_over_pi:
	.long	10680707                        ; 0xa2f983
	.long	7228996                         ; 0x6e4e44
	.long	1387004                         ; 0x1529fc
	.long	2578385                         ; 0x2757d1
	.long	16069853                        ; 0xf534dd
	.long	12639074                        ; 0xc0db62
	.long	9804092                         ; 0x95993c
	.long	4427841                         ; 0x439041
	.long	16666979                        ; 0xfe5163
	.long	11263675                        ; 0xabdebb
	.long	12935607                        ; 0xc561b7
	.long	2387514                         ; 0x246e3a
	.long	4345298                         ; 0x424dd2
	.long	14681673                        ; 0xe00649
	.long	3074569                         ; 0x2eea09
	.long	13734428                        ; 0xd1921c
	.long	16653803                        ; 0xfe1deb
	.long	1880361                         ; 0x1cb129
	.long	10960616                        ; 0xa73ee8
	.long	8533493                         ; 0x8235f5
	.long	3062596                         ; 0x2ebb44
	.long	8710556                         ; 0x84e99c
	.long	7349940                         ; 0x7026b4
	.long	6258241                         ; 0x5f7e41
	.long	3772886                         ; 0x3991d6
	.long	3769171                         ; 0x398353
	.long	3798172                         ; 0x39f49c
	.long	8675211                         ; 0x845f8b
	.long	12450088                        ; 0xbdf928
	.long	3874808                         ; 0x3b1ff8
	.long	9961438                         ; 0x97ffde
	.long	366607                          ; 0x5980f
	.long	15675153                        ; 0xef2f11
	.long	9132554                         ; 0x8b5a0a
	.long	7151469                         ; 0x6d1f6d
	.long	3571407                         ; 0x367ecf
	.long	2607881                         ; 0x27cb09
	.long	12013382                        ; 0xb74f46
	.long	4155038                         ; 0x3f669e
	.long	6285869                         ; 0x5fea2d
	.long	7677882                         ; 0x7527ba
	.long	13102053                        ; 0xc7ebe5
	.long	15825725                        ; 0xf17b3d
	.long	473591                          ; 0x739f7
	.long	9065106                         ; 0x8a5292
	.long	15363067                        ; 0xea6bfb
	.long	6271263                         ; 0x5fb11f
	.long	9264392                         ; 0x8d5d08
	.long	5636912                         ; 0x560330
	.long	4652155                         ; 0x46fc7b
	.long	7056368                         ; 0x6babf0
	.long	13614112                        ; 0xcfbc20
	.long	10155062                        ; 0x9af436
	.long	1944035                         ; 0x1da9e3
	.long	9527646                         ; 0x91615e
	.long	15080200                        ; 0xe61b08
	.long	6658437                         ; 0x659985
	.long	6231200                         ; 0x5f14a0
	.long	6832269                         ; 0x68408d
	.long	16767104                        ; 0xffd880
	.long	5075751                         ; 0x4d7327
	.long	3212806                         ; 0x310606
	.long	1398474                         ; 0x1556ca
	.long	7579849                         ; 0x73a8c9
	.long	6349435                         ; 0x60e27b
	.long	12618859                        ; 0xc08c6b

	.p2align	2, 0x0                          ; @_ZZ18__ieee754_rem_pio2dPdE8npio2_hw
__ZZ18__ieee754_rem_pio2dPdE8npio2_hw:
	.long	1073291771                      ; 0x3ff921fb
	.long	1074340347                      ; 0x400921fb
	.long	1074977148                      ; 0x4012d97c
	.long	1075388923                      ; 0x401921fb
	.long	1075800698                      ; 0x401f6a7a
	.long	1076025724                      ; 0x4022d97c
	.long	1076231611                      ; 0x4025fdbb
	.long	1076437499                      ; 0x402921fb
	.long	1076643386                      ; 0x402c463a
	.long	1076849274                      ; 0x402f6a7a
	.long	1076971356                      ; 0x4031475c
	.long	1077074300                      ; 0x4032d97c
	.long	1077177244                      ; 0x40346b9c
	.long	1077280187                      ; 0x4035fdbb
	.long	1077383131                      ; 0x40378fdb
	.long	1077486075                      ; 0x403921fb
	.long	1077589019                      ; 0x403ab41b
	.long	1077691962                      ; 0x403c463a
	.long	1077794906                      ; 0x403dd85a
	.long	1077897850                      ; 0x403f6a7a
	.long	1077968460                      ; 0x40407e4c
	.long	1078019932                      ; 0x4041475c
	.long	1078071404                      ; 0x4042106c
	.long	1078122876                      ; 0x4042d97c
	.long	1078174348                      ; 0x4043a28c
	.long	1078225820                      ; 0x40446b9c
	.long	1078277292                      ; 0x404534ac
	.long	1078328763                      ; 0x4045fdbb
	.long	1078380235                      ; 0x4046c6cb
	.long	1078431707                      ; 0x40478fdb
	.long	1078483179                      ; 0x404858eb
	.long	1078534651                      ; 0x404921fb

	.p2align	2, 0x0                          ; @_ZZ17__kernel_rem_pio2PdS_iiiPKiE7init_jk
__ZZ17__kernel_rem_pio2PdS_iiiPKiE7init_jk:
	.long	2                               ; 0x2
	.long	3                               ; 0x3
	.long	4                               ; 0x4
	.long	6                               ; 0x6

	.p2align	3, 0x0                          ; @_ZZ17__kernel_rem_pio2PdS_iiiPKiE4PIo2
__ZZ17__kernel_rem_pio2PdS_iiiPKiE4PIo2:
	.quad	0x3ff921fb40000000              ; double 1.5707962512969971
	.quad	0x3e74442d00000000              ; double 7.5497894158615964E-8
	.quad	0x3cf8469880000000              ; double 5.3903025299577648E-15
	.quad	0x3b78cc5160000000              ; double 3.2820034158079129E-22
	.quad	0x39f01b8380000000              ; double 1.2706557530806761E-29
	.quad	0x387a252040000000              ; double 1.2293330898111133E-36
	.quad	0x36e3822280000000              ; double 2.7337005381646456E-44
	.quad	0x3569f31d00000000              ; double 2.1674168387780482E-51

	.section	__TEXT,__cstring,cstring_literals
l_.str:                                 ; @.str
	.asciz	"%.17g %.17g %.17g\n"

.subsections_via_symbols
