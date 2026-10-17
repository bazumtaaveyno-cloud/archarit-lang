// ============================================================
// ARCHARIT — DIVISION 1: CORE LANGUAGE
// FILE: division1_core_tests.rs
// STATUS: COMPLETE DIVISION 1 TEST SUITE
// ============================================================

use crate::division1_core_checker::Checker;
use crate::division1_core_lexer::Lexer;
use crate::division1_core_parser::Parser;
use crate::division1_core_runtime::Runtime;

fn lex(source: &str) {
    Lexer::tokenize(source)
        .unwrap_or_else(|error| panic!("Lexer failed: {error}"));
}

fn parse(source: &str) {
    Parser::parse(source)
        .unwrap_or_else(|error| panic!("Parser failed: {error}"));
}

fn check(source: &str) {
    let program = Parser::parse(source)
        .unwrap_or_else(|error| panic!("Parser failed: {error}"));

    Checker::check(&program)
        .unwrap_or_else(|error| panic!("Checker failed: {error}"));
}

fn run(source: &str) {
    let program = Parser::parse(source)
        .unwrap_or_else(|error| panic!("Parser failed: {error}"));

    Checker::check(&program)
        .unwrap_or_else(|error| panic!("Checker failed: {error}"));

    Runtime::run(&program)
        .unwrap_or_else(|error| panic!("Runtime failed: {error}"));
}

fn assert_check_error(source: &str) {
    let program = Parser::parse(source)
        .unwrap_or_else(|error| panic!("Parser failed: {error}"));

    assert!(
        Checker::check(&program).is_err(),
        "Expected a type error for:
{source}"
    );
}

// ============================================================
// 1. LEXER TESTS
// ============================================================

#[cfg(test)]
mod lexer_tests {
    use super::*;
    use crate::division1_core_lexer::TokenKind;

    #[test]
    fn lexes_all_division_one_keywords() {
        let tokens = Lexer::tokenize(
            "val var let mut fun return if elif else match guard for while loop in break continue true false null Int f32 String Bool List Map Set defer",
        )
        .unwrap();

        let expected_count = 29;
        assert_eq!(tokens.len(), expected_count + 1);
    }

    #[test]
    fn lexes_int_and_f32() {
        let tokens = Lexer::tokenize("20 2.5").unwrap();

        assert_eq!(tokens[0].kind, TokenKind::Int(20));
        assert_eq!(tokens[1].kind, TokenKind::Float(2.5));
    }

    #[test]
    fn lexes_ranges() {
        let tokens = Lexer::tokenize("0..10 0..=10").unwrap();

        assert_eq!(tokens[1].kind, TokenKind::DotDot);
        assert_eq!(tokens[4].kind, TokenKind::DotDotEquals);
    }

    #[test]
    fn lexes_vector_operators() {
        let tokens = Lexer::tokenize("a .* b a ./ b a .+ b").unwrap();

        assert_eq!(tokens[1].kind, TokenKind::VectorStar);
        assert_eq!(tokens[4].kind, TokenKind::VectorSlash);
        assert_eq!(tokens[7].kind, TokenKind::VectorPlus);
    }

    #[test]
    fn lexes_string_interpolation() {
        let tokens = Lexer::tokenize(""Hello $name and ${2 + 2}"").unwrap();

        assert!(matches!(
            tokens[0].kind,
            crate::division1_core_lexer::TokenKind::InterpolatedText(_)
        ));
    }

    #[test]
    fn lexes_multiline_string() {
        let tokens = Lexer::tokenize(""""
line one
line two
"""").unwrap();

        assert_eq!(
            tokens[0].kind,
            TokenKind::Text("
line one
line two
".into())
        );
    }

    #[test]
    fn tracks_positions() {
        let tokens = Lexer::tokenize("val x
val y").unwrap();

        assert_eq!(tokens[0].line, 1);
        assert_eq!(tokens[0].column, 1);
        assert_eq!(tokens[3].line, 2);
        assert_eq!(tokens[3].column, 1);
    }

    #[test]
    fn rejects_unknown_character() {
        assert!(Lexer::tokenize("val @").is_err());
    }
}

// ============================================================
// 2. PARSER TESTS
// ============================================================

#[cfg(test)]
mod parser_tests {
    use super::*;

    #[test]
    fn parses_variables() {
        parse("val name: String = "Archarit"");
        parse("var age: Int = 20");
        parse("let mut score: f32 = 0");
        parse("val inferred = 10");
    }

    #[test]
    fn parses_functions() {
        parse("fun add(a: f32, b: f32): f32 { return a + b }");
        parse("fun greet(name: String): String { return "Hello $name" }");
        parse("val double = fun(value: Int): Int { return value * 2 }");
    }

    #[test]
    fn parses_both_if_forms() {
        parse("if age < 18 { print("kid") } elif age < 60 { print("adult") } else { print("elder") }");
        parse("if(age < 18) { print("kid") }");
    }

    #[test]
    fn parses_match() {
        parse(
            "match age { 0..17 => print("kid") 18..59 => print("adult") _ => print("other") }",
        );
    }

    #[test]
    fn parses_guard() {
        parse("guard age > 0 else { return }");
        parse("guard let user = maybeUser else { return }");
    }

    #[test]
    fn parses_loops() {
        parse("for i in 0..10 { if i == 5 { continue } if i == 9 { break } }");
        parse("while age < 30 { age += 1 }");
        parse("loop { break }");
    }

    #[test]
    fn parses_collections() {
        parse("val list: List<Int> = [1, 2, 3]");
        parse("val names = ["John", "Jane"]");
        parse("val user = { "name": "John", "age": 20 }");
        parse("val unique = {1, 2, 3, 2}");
    }

    #[test]
    fn parses_strings() {
        parse("val greeting = "Hello $name"");
        parse("val calculated = "2 + 2 = ${2 + 2}"");
        parse("val multiline = """
Line 1
Line 2
"""");
    }

    #[test]
    fn parses_vector_math() {
        parse("val multiplied = a .* b");
        parse("val added = a .+ b");
        parse("val divided = a ./ b");
        parse("val dotProduct = DOT(a, b)");
    }

    #[test]
    fn parses_file_io() {
        parse(
            "val file = open("data.txt") val contents = file.read() file.write("Hello Archarit") defer file.close()",
        );
    }

    #[test]
    fn parses_input() {
        parse("val answer = read("Continue? yes/no: ")");
    }

    #[test]
    fn rejects_invalid_map_key() {
        assert!(Parser::parse("val user = { 1: "one" }").is_err());
    }
}

// ============================================================
// 3. TYPE CHECKER TESTS
// ============================================================

#[cfg(test)]
mod checker_tests {
    use super::*;

    #[test]
    fn accepts_int_to_f32() {
        check("val score: f32 = 10");
    }

    #[test]
    fn rejects_f32_to_int() {
        assert_check_error("val count: Int = 2.5");
    }

    #[test]
    fn accepts_numeric_promotion() {
        check("val total = 1 + 2.5");
        check("val total = 2.5 + 1");
    }

    #[test]
    fn rejects_invalid_string_addition() {
        assert_check_error("val total = "a" + 1");
    }

    #[test]
    fn accepts_string_map() {
        check("val user = { "name": "John" }");
    }

    #[test]
    fn rejects_non_string_map_key() {
        assert_check_error("val user = { 1: "one" }");
    }

    #[test]
    fn accepts_vector_operations() {
        check("val result = [1, 2, 3] .* [4, 5, 6]");
        check("val result = [1, 2, 3] .+ [4, 5, 6]");
        check("val result = [1, 2, 3] ./ [4, 5, 6]");
    }

    #[test]
    fn rejects_immutable_assignment() {
        assert_check_error("val x = 1 x = 2");
    }

    #[test]
    fn accepts_mutable_assignment() {
        check("var x = 1 x = 2");
        check("let mut x = 1 x = 2");
    }

    #[test]
    fn accepts_valid_function_call() {
        check("fun add(a: f32, b: f32): f32 { return a + b } val total = add(1, 2.5)");
    }

    #[test]
    fn rejects_wrong_argument_count() {
        assert_check_error(
            "fun add(a: f32, b: f32): f32 { return a + b } val total = add(1)",
        );
    }

    #[test]
    fn accepts_bool_conditions() {
        check("if true { print("yes") }");
        check("while false { print("no") }");
        check("guard true else { return }");
    }

    #[test]
    fn rejects_non_bool_condition() {
        assert_check_error("if 1 { print("yes") }");
        assert_check_error("while 1 { print("no") }");
        assert_check_error("guard 1 else { return }");
    }
}

// ============================================================
// 4. RUNTIME TESTS
// ============================================================

#[cfg(test)]
mod runtime_tests {
    use super::*;

    #[test]
    fn runs_variables_and_print() {
        run("val name = "Archarit" print("Hello $name")");
    }

    #[test]
    fn runs_functions() {
        run(
            "fun add(a: f32, b: f32): f32 { return a + b } val total = add(1, 2.5) print(total)",
        );
    }

    #[test]
    fn runs_closures() {
        run(
            "val double = fun(value: Int): Int { return value * 2 } print(double(21))",
        );
    }

    #[test]
    fn runs_if_else() {
        run(
            "var age = 20 if age < 18 { print("kid") } elif age < 60 { print("adult") } else { print("elder") }",
        );
    }

    #[test]
    fn runs_c_style_if() {
        run("if(20 < 60) { print("adult") }");
    }

    #[test]
    fn runs_match_ranges() {
        run(
            "val age = 20 match age { 0..17 => print("kid") 18..59 => print("adult") _ => print("other") }",
        );
    }

    #[test]
    fn runs_guard() {
        run("val age = 20 guard age > 0 else { return } print("valid")");
    }

    #[test]
    fn runs_for_loop_with_break_and_continue() {
        run(
            "for i in 0..10 { if i == 5 { continue } if i == 9 { break } print(i) }",
        );
    }

    #[test]
    fn runs_while_loop() {
        run("var count = 0 while count < 3 { count += 1 print(count) }");
    }

    #[test]
    fn runs_loop_with_break() {
        run("loop { print("once") break }");
    }

    #[test]
    fn runs_list_operations() {
        run(
            "val list: List<Int> = [1, 2, 3] list.add(4) print(list.len()) print(list[0])",
        );
    }

    #[test]
    fn runs_map_operations() {
        run(
            "val user = { "name": "John", "age": 20 } print(user["name"]) user["age"] = 21 print(user["age"])",
        );
    }

    #[test]
    fn runs_set_deduplication() {
        run("val unique = {1, 2, 3, 2} print(unique.len())");
    }

    #[test]
    fn runs_string_methods() {
        run(
            "print("hello".upper()) print("HELLO".lower()) print("  hi  ".trim()) print("a,b,c".split(",")) print("hello".len())",
        );
    }

    #[test]
    fn runs_string_interpolation() {
        run(
            "val name = "Archarit" print("Hello $name") print("2 + 2 = ${2 + 2}")",
        );
    }

    #[test]
    fn runs_vector_math() {
        run(
            "val a = [1, 2, 3] val b = [4, 5, 6] print(a .* b) print(a .+ b) print(DOT(a, b))",
        );
    }

    #[test]
    fn runs_defer() {
        run("defer print("cleanup") print("working")");
    }

    #[test]
    fn runs_recursion() {
        run(
            "fun countdown(value: Int): Int { if value == 0 { return 0 } return countdown(value - 1) } print(countdown(3))",
        );
    }

    #[test]
    fn rejects_runtime_division_by_zero() {
        let program = Parser::parse("val result = 1 / 0").unwrap();

        assert!(Runtime::run(&program).is_err());
    }
}

// ============================================================
// 5. FULL DIVISION 1 ACCEPTANCE TEST
// ============================================================

#[cfg(test)]
mod acceptance_tests {
    use super::*;

    #[test]
    fn runs_complete_division_one_program() {
        let source = r#"
            val nameConst: String = "Archarit"
            var age: Int = 20
            let mut score: f32 = 0

            fun add(a: f32, b: f32): f32 {
                return a + b
            }

            if age < 18 {
                print("kid")
            } elif age < 60 {
                print("adult")
            } else {
                print("elder")
            }

            if(age < 18) {
                print("kid")
            }

            match age {
                0..17 => print("kid")
                18..59 => print("adult")
                _ => print("other")
            }

            guard age > 0 else {
                return
            }

            for i in 0..10 {
                if i == 5 {
                    continue
                }

                if i == 9 {
                    break
                }

                print(i)
            }

            var state = false
            val n = "yes"

            if n == "yes" {
                state = true
                print("true")
            } else {
                state = false
                print("false")
            }

            val list: List<Int> = [1, 2, 3]
            val names = ["John", "Jane"]
            val user = { "name": "John", "age": 20 }
            val unique = {1, 2, 3, 2}

            list.add(4)
            list.remove(0)

            print(list.len())
            print("John" in names)

            val greeting = "Hello $nameConst"
            val calculated = "2 + 2 = ${2 + 2}"

            val multiline = """
                Line 1
                Line 2
            """

            print("hello".upper())
            print("  hi  ".trim())
            print("a,b,c".split(","))

            val a = [1, 2, 3]
            val b = [4, 5, 6]

            val multiplied = a .* b
            val added = a .+ b
            val dotProduct = DOT(a, b)

            print(multiplied)
            print(added)
            print(dotProduct)

            val file = open("data.txt")
            val contents = file.read()
            file.write("Hello Archarit")
            defer file.close()

            print(add(1, 2.5))
        "#;

        run(source);
    }
}