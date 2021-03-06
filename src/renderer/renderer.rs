use std::fmt;

use abc::*;
use container::expression::*;
use container::template::*;
use renderer::Writer;
use {Arg, Args, VarContext};

use super::eval_expr;


pub fn text(context: &VarContext) -> RenderResult<String> {
    let mut buffer = String::new();
    render_text(&mut buffer, context, context.template().root.as_slice())?;
    Ok(buffer)
}


pub fn render_text<W: fmt::Write>(writer: &mut W, context: &VarContext, nodes: &[Node]) -> RenderResult<()> {
    for x in nodes {
        match *x {
            Node::Text(ref txt) => write!(writer, "{}", txt)?,
            Node::Mustache(ref mus) => render_mustache(writer, context, mus)?,
            Node::For(ref stmt) => render_for(writer, context, stmt)?,
            Node::If(ref stmt) => render_if(writer, context, stmt)?,
            Node::Block(ref stmt) => render_block(writer, context, stmt)?,
            Node::Include(ref expr) => render_include(writer, context, expr)?,
        }
    }
    Ok(())
}


pub fn render_mustache<W: fmt::Write>(writer: &mut W, context: &VarContext, mus: &Mustache) -> RenderResult<()> {
    render_expression(writer, context, &mus.expr)
}


pub fn render_expression<W: fmt::Write>(writer: &mut W, context: &VarContext, expr: &FullExpression) -> RenderResult<()> {
    let mut acc = eval_expr(context, &expr.expr)?;
    for filter in &expr.filters {
        acc = match *filter {
            FilterItem::Simple(ref id) => context.env().filter(id, context, acc)?,
        };
    }
    match acc {
        None => write!(writer, "#None")?,
        Some(acc) => acc.render(&mut Writer(writer))?,
    }
    Ok(())
}


pub fn render_for<W: fmt::Write>(writer: &mut W, context: &VarContext, stmt: &ForStatement) -> RenderResult<()> {
    // FIXME implement instead: expression(&stmt.begin.expression, context)
    if let Some(value) = eval_expr(context, &stmt.expression.expr)? {
        if let Some(iterable) = value.try_as_iterable() {
            use container::cycle::LoopState;
            // TODO the "last" marker in a loop
            let mut state = LoopState::new(false);
            for v in iterable.ivalues() {
                {
                    let local_scope: Args = hashmap! {
                        stmt.value_var.as_str().into() => v.to_ref(),
                        "loop".into() => Arg::Ref(&state),
                    };
                    render_text(writer, &context.nested_scope(&local_scope), &stmt.block)?;
                }
                state = state.next(false);
            }
        }
    };
    Ok(())
}


pub fn render_if<W: fmt::Write>(writer: &mut W, context: &VarContext, stmt: &IfStatement) -> RenderResult<()> {
    for branch in &stmt.if_branches {
        if let Some(res) = eval_expr(context, &branch.expr.expr)? {
            if res.to_bool() {
                render_text(writer, context, &branch.block)?;
                return Ok(());
            }
        }
    }
    if let Some(ref branch) = stmt.else_branch {
        render_text(writer, context, branch)?;
    }
    Ok(())
}


pub fn render_block<W: fmt::Write>(writer: &mut W, context: &VarContext, name: &str) -> RenderResult<()> {
    for template in context.global().stack() {
        if let Some(block) = template.blocks.get(name) {
            render_text(writer, context, block)?;
            break;
        };
    }
    Ok(())
}


pub fn render_include<W: fmt::Write>(writer: &mut W, context: &VarContext, expr: &FullExpression) -> RenderResult<()> {
    let name = eval_expr(context, &expr.expr)?
        .ok_or_else(|| LoadError::BadName("Can't evaluate name (None result)".into()))?;
    let name = name.try_as_string()
        .ok_or_else(|| LoadError::BadName("Name is not string".into()))?;
    let template = context.global().env().get_template(&name)?;
    // FIXME Base context
    // render_text(writer, &context.global().top_scope(), template.root.as_slice())
    // Current scope context
    render_text(writer, context, template.root.as_slice())
}
