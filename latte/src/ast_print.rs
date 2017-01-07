use std::fmt;

use ast::*;

static FERR: &'static str = "Unexpected format error";

pub fn print_code(p: &Program) {
    print!("{}", p);
}

impl fmt::Display for Program {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.print0(f);
        Ok(())
    }
}

// Display adds indentation to standard fmt::Display
trait Display {
    fn print0(&self, dst: &mut fmt::Write) {
        self.print(&String::new(), dst);
    }

    fn print(&self, indent: &String, dst: &mut fmt::Write);

    fn next_indent(indent: &String) -> String {
        format!("\t{}", indent)
    }
}

impl Display for Program {
    fn print(&self, indent: &String, dst: &mut fmt::Write) {
        let Program(ref defs) = *self;
        defs.print(indent, dst);
    }
}

impl fmt::Display for Def {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.print0(f);
        Ok(())
    }
}

impl Display for Def {
    fn print(&self, indent: &String, dst: &mut fmt::Write) {
        match *self {
            Def::DFunc(ref func) => func.print(indent, dst),
            Def::DClass(ref class) => class.print(indent, dst),
        }
    }
}

impl Display for Class {
    fn print(&self, indent: &String, dst: &mut fmt::Write) {
        let inner_indent = Self::next_indent(indent);
        let extends = match self.superclass {
            Some(ref superclass) => format!("extends {} ", superclass),
            None => format!(""),
        };
        writeln!(dst, "{}class {} {}{}", indent, self.name, extends, '{').expect(FERR);
        for var in &self.vars {
            writeln!(dst, "{}{};", &inner_indent, var).expect(FERR);
        }
        if !self.vars.is_empty() && !self.methods.is_empty() {
            writeln!(dst, "").expect(FERR);
        }
        self.methods.print(&inner_indent, dst);
        writeln!(dst, "{}{}", indent, '}').expect(FERR);
    }
}

impl Display for Func {
    fn print(&self, indent: &String, dst: &mut fmt::Write) {
        let inner_indent = Self::next_indent(indent);
        writeln!(dst,
                 "{}{} {}({}) {}",
                 indent,
                 self.ret_type,
                 self.ident,
                 print_vec(&self.args),
                 '{')
            .expect(FERR);
        self.body.print(&inner_indent, dst);
        writeln!(dst, "{}{}", indent, '}').expect(FERR);
    }
}


impl fmt::Display for Stmt {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.print0(f);
        Ok(())
    }
}

impl Display for Stmt {
    fn print(&self, indent: &String, dst: &mut fmt::Write) {
        let inner_indent = Self::next_indent(indent);
        match *self {
            Stmt::SEmpty => {}
            Stmt::SBlock(ref stmts) => {
                writeln!(dst, "{}{}", indent, '{').expect(FERR);
                stmts.print(&inner_indent, dst);
                writeln!(dst, "{}{}", indent, '}').expect(FERR);
            }
            Stmt::SDecl(ref t, ref inits) => {
                writeln!(dst, "{}{} {};", indent, t, print_vec(inits)).expect(FERR)
            }
            Stmt::SAssign(ref i, ref e) => writeln!(dst, "{}{} = {};", indent, i, e).expect(FERR),
            Stmt::SInc(ref i) => writeln!(dst, "{}{}++;", indent, i).expect(FERR),
            Stmt::SDec(ref i) => writeln!(dst, "{}{}--;", indent, i).expect(FERR),
            Stmt::SReturnE(ref e) => writeln!(dst, "{}return {};", indent, e).expect(FERR),
            Stmt::SReturn => writeln!(dst, "{}return;", indent).expect(FERR),
            Stmt::SExpr(ref e) => writeln!(dst, "{}{};", indent, e).expect(FERR),
            Stmt::SIf(ref cond, ref stmt) => {
                writeln!(dst, "{}if ({}) {}", indent, cond, '{').expect(FERR);
                match **stmt {
                    Stmt::SBlock(ref stmts) => stmts.print(&inner_indent, dst),
                    _ => stmt.print(&inner_indent, dst),
                };
                writeln!(dst, "{}{}", indent, '}').expect(FERR);
            }
            Stmt::SIfElse(ref cond, ref if_t, ref if_f) => {
                writeln!(dst, "{}if ({}) {}", indent, cond, '{').expect(FERR);
                match **if_t {
                    Stmt::SBlock(ref stmts) => stmts.print(&inner_indent, dst),
                    _ => if_t.print(&inner_indent, dst),	
                };
                writeln!(dst, "{}{} else {}", indent, '}', '{').expect(FERR);
                match **if_f {
                    Stmt::SBlock(ref stmts) => stmts.print(&inner_indent, dst),
                    _ => if_f.print(&inner_indent, dst),	
                };
                writeln!(dst, "{}{}", indent, '}').expect(FERR);
            }
            Stmt::SWhile(ref cond, ref stmt) => {
                writeln!(dst, "{}while ({}) {}", indent, cond, '{').expect(FERR);
                match **stmt {
                    Stmt::SBlock(ref stmts) => stmts.print(&inner_indent, dst),
                    _ => stmt.print(&inner_indent, dst),	
                };
                writeln!(dst, "{}{}", indent, '}').expect(FERR);
            }
        };
    }
}

impl<T> Display for Vec<T>
    where T: Display
{
    fn print(&self, indent: &String, dst: &mut fmt::Write) {
        for x in self {
            x.print(indent, dst);
        }
    }
}

impl fmt::Display for Var {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {}", self.t, self.ident)
    }
}

impl fmt::Display for VarDecl {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match *self {
            VarDecl::Init(_, ref name, ref e) => format!("{} = {}", name, e),
            VarDecl::NoInit(_, ref name) => format!("{}", name),
        };
        write!(f, "{}", s)
    }
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match *self {
            Expr::EVar(ref i) => format!("{}", i),
            Expr::ELit(ref i) => format!("{}", i),
            Expr::ECall(ref f, ref args) => format!("{}({})", f, print_vec(args)),
            Expr::ENeg(ref e) => format!("-{}", *e),
            Expr::ENot(ref e) => format!("!{}", *e),
            Expr::EBinOp(ref lhs, ref op, ref rhs) => format!("({} {} {})", *lhs, op, *rhs),
        };
        write!(f, "{}", s)
    }
}

impl fmt::Display for Lit {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match *self {
            Lit::LInt(x) => format!("{}", x),
            Lit::LTrue => format!("true"),
            Lit::LFalse => format!("false"),
            Lit::LString(ref s) => format!("\"{}\"", s),
            Lit::LNull => format!("null"),
        };
        write!(f, "{}", s)
    }
}

impl fmt::Display for FieldGet {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.ident)?;
        if let Some(ref field) = self.field {
            write!(f, ".{}", field)?;
        }
        Ok(())
    }
}

impl fmt::Display for Ident {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let Ident(ref s) = *self;
        write!(f, "{}", s)
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match *self {
            Type::TInt => format!("int"),
            Type::TString => format!("string"),
            Type::TBool => format!("boolean"),
            Type::TVoid => format!("void"),
            Type::TFunc(ref args, ref ret_type) => format!("({}) -> {}", print_vec(args), ret_type),
            Type::TObject(ref cname) => format!("{}", cname),
            Type::TNull => format!("<null_type>"),
        };
        write!(f, "{}", s)
    }
}

impl fmt::Display for Operator {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match *self {
            Operator::OpAdd => "+",
            Operator::OpSub => "-",
            Operator::OpMul => "*",
            Operator::OpDiv => "/",
            Operator::OpMod => "%",
            Operator::OpLess => "<",
            Operator::OpGreater => ">",
            Operator::OpLessE => "<=",
            Operator::OpGreaterE => ">=",
            Operator::OpEq => "==",
            Operator::OpNEq => "!=",
            Operator::OpAnd => "&&",
            Operator::OpOr => "||",
        };
        write!(f, "{}", s)
    }
}

fn print_vec<T>(vec: &Vec<T>) -> String
    where T: fmt::Display
{
    let mut s = String::new();
    for x in vec {
        if !s.is_empty() {
            s.push_str(", ");
        }
        s.push_str(format!("{}", x).as_str());
    }
    s
}
