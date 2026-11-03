use crate::ast::{Expr, ExprKind, Stmt, StmtKind};
use std::fmt::Write;

const RUNTIME: &str = r#"
#include <cmath>
#include <cstdint>
#include <functional>
#include <iostream>
#include <limits>
#include <map>
#include <memory>
#include <stdexcept>
#include <string>
#include <variant>
#include <vector>
#include <boost/multiprecision/cpp_int.hpp>

struct Value;
using Integer = boost::multiprecision::cpp_int;
using List = std::vector<Value>;
using Dict = std::map<std::string, Value>;
using ListPtr = std::shared_ptr<List>;
using DictPtr = std::shared_ptr<Dict>;

struct Value {
    std::variant<std::monostate, Integer, double, bool, std::string, ListPtr, DictPtr> data;
    Value() = default;
    Value(Integer value) : data(std::move(value)) {}
    Value(std::int64_t value) : data(Integer(value)) {}
    Value(double value) : data(value) {}
    Value(bool value) : data(value) {}
    Value(const char* value) : data(std::string(value)) {}
    Value(std::string value) : data(std::move(value)) {}
    static Value list(List value) { Value result; result.data = std::make_shared<List>(std::move(value)); return result; }
    static Value dict(Dict value) { Value result; result.data = std::make_shared<Dict>(std::move(value)); return result; }
};

static std::string text(const Value& value);
static double number(const Value& value) {
    if (auto item = std::get_if<Integer>(&value.data)) return item->convert_to<double>();
    if (auto item = std::get_if<double>(&value.data)) return *item;
    if (auto item = std::get_if<bool>(&value.data)) return *item ? 1.0 : 0.0;
    if (auto item = std::get_if<std::string>(&value.data)) {
        try { return std::stod(*item); } catch (...) { throw std::runtime_error("invalid numeric conversion"); }
    }
    throw std::runtime_error("expected a number");
}
static bool truthy(const Value& value) {
    if (std::holds_alternative<std::monostate>(value.data)) return false;
    if (auto item = std::get_if<bool>(&value.data)) return *item;
    if (auto item = std::get_if<Integer>(&value.data)) return *item != 0;
    if (auto item = std::get_if<double>(&value.data)) return *item != 0.0;
    if (auto item = std::get_if<std::string>(&value.data)) return !item->empty();
    if (auto item = std::get_if<ListPtr>(&value.data)) return !(*item)->empty();
    return !std::get<DictPtr>(value.data)->empty();
}
static Value logical_and(const Value& left, const std::function<Value()>& right) {
    return truthy(left) ? right() : left;
}
static Value logical_or(const Value& left, const std::function<Value()>& right) {
    return truthy(left) ? left : right();
}
static Integer integer(const Value& value);
static std::string text(const Value& value) {
    if (std::holds_alternative<std::monostate>(value.data)) return "None";
    if (auto item = std::get_if<bool>(&value.data)) return *item ? "True" : "False";
    if (auto item = std::get_if<Integer>(&value.data)) return item->convert_to<std::string>();
    if (auto item = std::get_if<double>(&value.data)) {
        std::string result = std::to_string(*item);
        while (result.size() > 1 && result.back() == '0') result.pop_back();
        if (result.back() == '.') result.pop_back();
        return result;
    }
    if (auto item = std::get_if<std::string>(&value.data)) return *item;
    if (auto item = std::get_if<ListPtr>(&value.data)) {
        std::string result = "[";
        for (std::size_t i = 0; i < (*item)->size(); ++i) {
            if (i) result += ", ";
            result += text((**item)[i]);
        }
        return result + "]";
    }
    std::string result = "{";
    bool first = true;
    for (const auto& [key, item] : *std::get<DictPtr>(value.data)) {
        if (!first) result += ", ";
        first = false;
        result += "'" + key + "': " + text(item);
    }
    return result + "}";
}
static Value add(Value a, Value b) {
    if (auto left = std::get_if<std::string>(&a.data)) return Value(*left + text(b));
    if (auto left = std::get_if<ListPtr>(&a.data)) {
        if (auto right = std::get_if<ListPtr>(&b.data)) {
            List result = **left; result.insert(result.end(), (*right)->begin(), (*right)->end());
            return Value::list(std::move(result));
        }
    }
    if (std::holds_alternative<double>(a.data) || std::holds_alternative<double>(b.data))
        return Value(number(a) + number(b));
    return Value(integer(a) + integer(b));
}
static Value subtract(Value a, Value b) {
    if (std::holds_alternative<double>(a.data) || std::holds_alternative<double>(b.data))
        return Value(number(a) - number(b));
    return Value(integer(a) - integer(b));
}
static Value multiply(Value a, Value b) {
    if (std::holds_alternative<double>(a.data) || std::holds_alternative<double>(b.data))
        return Value(number(a) * number(b));
    return Value(integer(a) * integer(b));
}
static Value divide(Value a, Value b) {
    double divisor = number(b);
    if (divisor == 0.0) throw std::runtime_error("division by zero");
    return Value(number(a) / divisor);
}
static Value modulo(Value a, Value b) {
    if (std::holds_alternative<double>(a.data) || std::holds_alternative<double>(b.data)) {
        double divisor = number(b);
        if (divisor == 0.0) throw std::runtime_error("modulo by zero");
        return Value(std::fmod(number(a), divisor));
    }
    auto dividend = integer(a);
    auto divisor = integer(b);
    if (divisor == 0) throw std::runtime_error("modulo by zero");
    return Value(dividend % divisor);
}
static Value power(Value a, Value b) { return Value(std::pow(number(a), number(b))); }
static Value negate(Value value) {
    if (std::holds_alternative<double>(value.data)) return Value(-number(value));
    return Value(-integer(value));
}
static bool equal(const Value& a, const Value& b) {
    auto numeric = [](const Value& value) {
        return std::holds_alternative<Integer>(value.data) || std::holds_alternative<double>(value.data);
    };
    if (std::holds_alternative<Integer>(a.data) && std::holds_alternative<Integer>(b.data))
        return std::get<Integer>(a.data) == std::get<Integer>(b.data);
    if (numeric(a) && numeric(b))
        return number(a) == number(b);
    if (a.data.index() != b.data.index()) return false;
    if (std::holds_alternative<std::monostate>(a.data)) return true;
    if (auto left = std::get_if<bool>(&a.data)) return *left == std::get<bool>(b.data);
    if (auto left = std::get_if<std::string>(&a.data)) return *left == std::get<std::string>(b.data);
    if (auto left = std::get_if<ListPtr>(&a.data)) {
        const auto& right = *std::get<ListPtr>(b.data);
        if ((*left)->size() != right.size()) return false;
        for (std::size_t i = 0; i < right.size(); ++i)
            if (!equal((**left)[i], right[i])) return false;
        return true;
    }
    if (auto left = std::get_if<DictPtr>(&a.data)) {
        const auto& right = *std::get<DictPtr>(b.data);
        if ((*left)->size() != right.size()) return false;
        for (const auto& [key, value] : **left) {
            auto found = right.find(key);
            if (found == right.end() || !equal(value, found->second)) return false;
        }
        return true;
    }
    return number(a) == number(b);
}
static bool less(const Value& a, const Value& b) {
    if (auto left = std::get_if<std::string>(&a.data)) {
        if (auto right = std::get_if<std::string>(&b.data)) return *left < *right;
    }
    if (auto left = std::get_if<Integer>(&a.data)) {
        if (auto right = std::get_if<Integer>(&b.data)) return *left < *right;
    }
    return number(a) < number(b);
}
static bool contains(const Value& container, const Value& item) {
    if (auto list = std::get_if<ListPtr>(&container.data))
        for (const auto& value : **list) if (equal(value, item)) return true;
    if (auto dict = std::get_if<DictPtr>(&container.data))
        return (*dict)->contains(text(item));
    if (auto string = std::get_if<std::string>(&container.data))
        return string->find(text(item)) != std::string::npos;
    throw std::runtime_error("right operand of `in` is not iterable");
}
static std::vector<Value> iterate(const Value& value) {
    if (auto list = std::get_if<ListPtr>(&value.data)) return **list;
    if (auto dict = std::get_if<DictPtr>(&value.data)) {
        std::vector<Value> result;
        for (const auto& [key, ignored] : **dict) { (void)ignored; result.emplace_back(key); }
        return result;
    }
    if (auto string = std::get_if<std::string>(&value.data)) {
        std::vector<Value> result;
        for (char ch : *string) result.emplace_back(std::string(1, ch));
        return result;
    }
    throw std::runtime_error("value is not iterable");
}
static Integer integer(const Value& value) {
    if (auto item = std::get_if<Integer>(&value.data)) return *item;
    if (auto item = std::get_if<bool>(&value.data)) return *item ? 1 : 0;
    if (auto item = std::get_if<std::string>(&value.data)) {
        try { return Integer(*item); }
        catch (const std::exception&) { return Integer(number(value)); }
    }
    return Integer(number(value));
}
static std::int64_t bounded_integer(const Value& value) {
    auto result = integer(value);
    if (result < std::numeric_limits<std::int64_t>::min() ||
        result > std::numeric_limits<std::int64_t>::max())
        throw std::runtime_error("integer is out of range");
    return result.convert_to<std::int64_t>();
}
static Value index_value(const Value& value, const Value& subscript) {
    if (auto list = std::get_if<ListPtr>(&value.data)) {
        auto index = bounded_integer(subscript);
        auto size = static_cast<std::int64_t>((*list)->size());
        if (index < 0) index += size;
        if (index < 0 || index >= size) throw std::runtime_error("list index out of range");
        return (**list)[static_cast<std::size_t>(index)];
    }
    if (auto string = std::get_if<std::string>(&value.data)) {
        auto index = bounded_integer(subscript);
        auto size = static_cast<std::int64_t>(string->size());
        if (index < 0) index += size;
        if (index < 0 || index >= size) throw std::runtime_error("string index out of range");
        return Value(std::string(1, (*string)[static_cast<std::size_t>(index)]));
    }
    if (auto dict = std::get_if<DictPtr>(&value.data)) {
        auto found = (*dict)->find(text(subscript));
        if (found == (*dict)->end()) throw std::runtime_error("dictionary key not found");
        return found->second;
    }
    throw std::runtime_error("value does not support indexing");
}
static Value slice_value(const Value& value, Value start, Value end, Value step) {
    auto values = iterate(value);
    auto length = static_cast<std::int64_t>(values.size());
    auto first_default = std::holds_alternative<std::monostate>(start.data);
    auto last_default = std::holds_alternative<std::monostate>(end.data);
    auto stride = std::holds_alternative<std::monostate>(step.data) ? 1 : bounded_integer(step);
    if (stride == 0) throw std::runtime_error("slice step cannot be zero");
    auto first = first_default ? (stride > 0 ? 0 : length - 1) : bounded_integer(start);
    auto last = last_default ? (stride > 0 ? length : -1) : bounded_integer(end);
    if (!first_default && first < 0) first += length;
    if (!last_default && last < 0) last += length;
    if (stride > 0) {
        first = std::clamp(first, std::int64_t{0}, length);
        last = std::clamp(last, std::int64_t{0}, length);
    } else {
        first = std::clamp(first, std::int64_t{-1}, length - 1);
        last = std::clamp(last, std::int64_t{-1}, length - 1);
    }
    List result;
    if (stride > 0) for (auto i = first; i < last; i += stride) result.push_back(values[static_cast<std::size_t>(i)]);
    else for (auto i = first; i > last; i += stride) result.push_back(values[static_cast<std::size_t>(i)]);
    if (std::holds_alternative<std::string>(value.data)) {
        std::string joined; for (const auto& item : result) joined += text(item);
        return Value(std::move(joined));
    }
    return Value::list(std::move(result));
}
static Value get(const std::map<std::string, Value>& env, const std::string& name) {
    auto found = env.find(name);
    if (found == env.end()) throw std::runtime_error("name `" + name + "` is not defined");
    return found->second;
}
static void set(std::map<std::string, Value>& env, const std::string& name, Value value) { env[name] = std::move(value); }
static Value arg(const std::vector<Value>& args, std::size_t index) {
    if (index >= args.size()) throw std::runtime_error("missing function argument");
    return args[index];
}
static Value builtin(const std::string& name, const std::vector<Value>& args) {
    if (name == "print") {
        for (std::size_t i = 0; i < args.size(); ++i) { if (i) std::cout << ' '; std::cout << text(args[i]); }
        std::cout << '\n'; return Value();
    }
    if (name == "input") {
        if (!args.empty()) std::cout << text(args[0]);
        std::string result; if (!std::getline(std::cin, result)) return Value();
        return Value(std::move(result));
    }
    if (name == "len") {
        if (args.size() != 1) throw std::runtime_error("len() expects one argument");
        if (auto string = std::get_if<std::string>(&args[0].data)) return Value(static_cast<std::int64_t>(string->size()));
        if (auto list = std::get_if<ListPtr>(&args[0].data)) return Value(static_cast<std::int64_t>((*list)->size()));
        if (auto dict = std::get_if<DictPtr>(&args[0].data)) return Value(static_cast<std::int64_t>((*dict)->size()));
        throw std::runtime_error("len() argument has no length");
    }
    if (name == "range") {
        if (args.empty() || args.size() > 3) throw std::runtime_error("range() expects one to three arguments");
        auto start = args.size() == 1 ? Integer{0} : integer(args[0]);
        auto stop = args.size() == 1 ? integer(args[0]) : integer(args[1]);
        auto step = args.size() == 3 ? integer(args[2]) : Integer{1};
        if (step == 0) throw std::runtime_error("range() step cannot be zero");
        List result;
        if (step > 0) for (auto i = start; i < stop; i += step) result.emplace_back(i);
        else for (auto i = start; i > stop; i += step) result.emplace_back(i);
        return Value::list(std::move(result));
    }
    if (args.size() != 1) throw std::runtime_error(name + "() expects one argument");
    if (name == "str") return Value(text(args[0]));
    if (name == "int") return Value(integer(args[0]));
    if (name == "float") return Value(number(args[0]));
    if (name == "bool") return Value(truthy(args[0]));
    if (name == "type") {
        const auto& data = args[0].data;
        if (std::holds_alternative<std::monostate>(data)) return Value("NoneType");
        if (std::holds_alternative<bool>(data)) return Value("bool");
        if (std::holds_alternative<Integer>(data)) return Value("int");
        if (std::holds_alternative<double>(data)) return Value("float");
        if (std::holds_alternative<std::string>(data)) return Value("str");
        if (std::holds_alternative<ListPtr>(data)) return Value("list");
        return Value("dict");
    }
    throw std::runtime_error("unknown builtin `" + name + "`");
}
static Value integer_literal(const std::string& value) {
    return Value(Integer(value));
}
"#;

pub fn generate(statements: &[Stmt]) -> String {
    let mut output = String::from(RUNTIME);
    output.insert_str(
        output.find("#include <cmath>").unwrap(),
        "#include <algorithm>\n",
    );

    let functions: Vec<_> = statements
        .iter()
        .filter_map(|statement| match &statement.kind {
            StmtKind::Function {
                name,
                parameters,
                body,
            } => Some((name.as_str(), parameters, body)),
            _ => None,
        })
        .collect();
    for (name, _, _) in &functions {
        let _ = writeln!(
            output,
            "static Value pyra_fn_{}(const std::vector<Value>&);",
            name
        );
    }
    for (name, parameters, body) in &functions {
        let _ = writeln!(
            output,
            "static Value pyra_fn_{}(const std::vector<Value>& args) {{",
            name
        );
        output.push_str("    if (args.size() != ");
        let _ = writeln!(
            output,
            "{}) throw std::runtime_error(\"function `{}` received the wrong number of arguments\");",
            parameters.len(),
            name
        );
        output.push_str("    std::map<std::string, Value> env;\n");
        for (index, parameter) in parameters.iter().enumerate() {
            let _ = writeln!(
                output,
                "    set(env, {}, args[{}]);",
                cpp_string(parameter),
                index
            );
        }
        emit_statements(body, &mut output, 1);
        output.push_str("    return Value();\n}\n");
    }
    output.push_str("int main() {\n    try {\n        std::map<std::string, Value> env;\n");
    for statement in statements {
        if !matches!(statement.kind, StmtKind::Function { .. }) {
            emit_statement(statement, &mut output, 2);
        }
    }
    output.push_str("        return 0;\n    } catch (const std::exception& error) {\n        std::cerr << \"runtime error: \" << error.what() << '\\n';\n        return 1;\n    }\n}\n");
    output
}

fn emit_statements(statements: &[Stmt], output: &mut String, depth: usize) {
    for statement in statements {
        emit_statement(statement, output, depth);
    }
}

fn emit_statement(statement: &Stmt, output: &mut String, depth: usize) {
    let indent = "    ".repeat(depth);
    match &statement.kind {
        StmtKind::Expr(expression) => {
            let _ = writeln!(output, "{indent}{};", expression_code(expression));
        }
        StmtKind::Assign {
            name,
            operator,
            value,
        } => {
            let value = expression_code(value);
            let value = match operator.as_str() {
                "+=" => format!("add(get(env, {}), {value})", cpp_string(name)),
                "-=" => format!("subtract(get(env, {}), {value})", cpp_string(name)),
                "*=" => format!("multiply(get(env, {}), {value})", cpp_string(name)),
                "/=" => format!("divide(get(env, {}), {value})", cpp_string(name)),
                "%=" => format!("modulo(get(env, {}), {value})", cpp_string(name)),
                _ => value,
            };
            let _ = writeln!(output, "{indent}set(env, {}, {value});", cpp_string(name));
        }
        StmtKind::If {
            branches,
            otherwise,
        } => {
            for (index, (condition, body)) in branches.iter().enumerate() {
                let keyword = if index == 0 { "if" } else { "else if" };
                let _ = writeln!(
                    output,
                    "{indent}{keyword} (truthy({})) {{",
                    expression_code(condition)
                );
                emit_statements(body, output, depth + 1);
                let _ = writeln!(output, "{indent}}}");
            }
            if !otherwise.is_empty() {
                let _ = writeln!(output, "{indent}else {{");
                emit_statements(otherwise, output, depth + 1);
                let _ = writeln!(output, "{indent}}}");
            }
        }
        StmtKind::While { condition, body } => {
            let _ = writeln!(
                output,
                "{indent}while (truthy({})) {{",
                expression_code(condition)
            );
            emit_statements(body, output, depth + 1);
            let _ = writeln!(output, "{indent}}}");
        }
        StmtKind::For {
            name,
            iterable,
            body,
        } => {
            let _ = writeln!(
                output,
                "{indent}for (const auto& item : iterate({})) {{",
                expression_code(iterable)
            );
            let _ = writeln!(output, "{indent}    set(env, {}, item);", cpp_string(name));
            emit_statements(body, output, depth + 1);
            let _ = writeln!(output, "{indent}}}");
        }
        StmtKind::Function { .. } => {}
        StmtKind::Return(value) => {
            let expression = value
                .as_ref()
                .map(expression_code)
                .unwrap_or_else(|| "Value()".into());
            let _ = writeln!(output, "{indent}return {expression};");
        }
        StmtKind::Break => {
            let _ = writeln!(output, "{indent}break;");
        }
        StmtKind::Continue => {
            let _ = writeln!(output, "{indent}continue;");
        }
    }
}

fn expression_code(expression: &Expr) -> String {
    match &expression.kind {
        ExprKind::None => "Value()".into(),
        ExprKind::Bool(value) => format!("Value({value})"),
        ExprKind::Number(value) if value.contains('.') => format!("Value({value})"),
        ExprKind::Number(value) => format!("integer_literal({})", cpp_string(value)),
        ExprKind::String(value) => format!("Value({})", cpp_string(value)),
        ExprKind::Name(name) => format!("get(env, {})", cpp_string(name)),
        ExprKind::List(items) => format!(
            "Value::list(std::vector<Value>{{{}}})",
            items
                .iter()
                .map(expression_code)
                .collect::<Vec<_>>()
                .join(", ")
        ),
        ExprKind::Dict(items) => format!(
            "Value::dict(std::map<std::string, Value>{{{}}})",
            items
                .iter()
                .map(|(key, value)| format!(
                    "{{text({}), {}}}",
                    expression_code(key),
                    expression_code(value)
                ))
                .collect::<Vec<_>>()
                .join(", ")
        ),
        ExprKind::Unary { operator, operand } => match operator.as_str() {
            "-" => format!("negate({})", expression_code(operand)),
            "+" => format!("Value(number({}))", expression_code(operand)),
            "not" => format!("Value(!truthy({}))", expression_code(operand)),
            _ => unreachable!("parser only creates supported unary operators"),
        },
        ExprKind::Binary {
            left,
            operator,
            right,
        } => {
            let left = expression_code(left);
            let right = expression_code(right);
            match operator.as_str() {
                "and" => format!("logical_and({left}, [&]() {{ return {right}; }})"),
                "or" => format!("logical_or({left}, [&]() {{ return {right}; }})"),
                "+" => sequenced_binary(&left, &right, "add(left_value, right_value)"),
                "-" => sequenced_binary(&left, &right, "subtract(left_value, right_value)"),
                "*" => sequenced_binary(&left, &right, "multiply(left_value, right_value)"),
                "/" => sequenced_binary(&left, &right, "divide(left_value, right_value)"),
                "%" => sequenced_binary(&left, &right, "modulo(left_value, right_value)"),
                "**" => sequenced_binary(&left, &right, "power(left_value, right_value)"),
                "==" => sequenced_binary(&left, &right, "Value(equal(left_value, right_value))"),
                "!=" => sequenced_binary(&left, &right, "Value(!equal(left_value, right_value))"),
                "<" => sequenced_binary(&left, &right, "Value(less(left_value, right_value))"),
                ">" => sequenced_binary(&left, &right, "Value(less(right_value, left_value))"),
                "<=" => sequenced_binary(&left, &right, "Value(!less(right_value, left_value))"),
                ">=" => sequenced_binary(&left, &right, "Value(!less(left_value, right_value))"),
                "in" => sequenced_binary(&left, &right, "Value(contains(right_value, left_value))"),
                _ => unreachable!("parser only creates supported binary operators"),
            }
        }
        ExprKind::Call { callee, arguments } => {
            let ExprKind::Name(name) = &callee.kind else {
                unreachable!("semantic analysis rejects non-name calls")
            };
            let arguments = arguments
                .iter()
                .map(expression_code)
                .collect::<Vec<_>>()
                .join(", ");
            let vector = format!("std::vector<Value>{{{arguments}}}");
            if matches!(
                name.as_str(),
                "print" | "input" | "len" | "range" | "str" | "int" | "float" | "bool" | "type"
            ) {
                format!("builtin({}, {vector})", cpp_string(name))
            } else {
                format!("pyra_fn_{name}({vector})")
            }
        }
        ExprKind::Index { value, index } => sequenced_binary(
            &expression_code(value),
            &expression_code(index),
            "index_value(left_value, right_value)",
        ),
        ExprKind::Slice {
            value,
            start,
            end,
            step,
        } => format!(
            "([&]() {{ auto slice_value_input = {}; auto slice_start = {}; auto slice_end = {}; auto slice_step = {}; return slice_value(slice_value_input, slice_start, slice_end, slice_step); }}())",
            expression_code(value),
            optional_expr(start),
            optional_expr(end),
            optional_expr(step)
        ),
    }
}

fn optional_expr(expression: &Option<Box<Expr>>) -> String {
    expression
        .as_ref()
        .map(|expression| expression_code(expression))
        .unwrap_or_else(|| "Value()".into())
}

fn sequenced_binary(left: &str, right: &str, operation: &str) -> String {
    format!(
        "([&]() {{ auto left_value = {left}; auto right_value = {right}; return {operation}; }}())"
    )
}

fn cpp_string(value: &str) -> String {
    let mut result = String::from("\"");
    for ch in value.chars() {
        match ch {
            '\\' => result.push_str("\\\\"),
            '"' => result.push_str("\\\""),
            '\n' => result.push_str("\\n"),
            '\r' => result.push_str("\\r"),
            '\t' => result.push_str("\\t"),
            '\0' => result.push_str("\\0"),
            ch if ch.is_control() => {
                let _ = write!(result, "\\u{:04x}", ch as u32);
            }
            ch => result.push(ch),
        }
    }
    result.push('"');
    result
}
