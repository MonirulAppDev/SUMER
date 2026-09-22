//! Top-level declaration parsing (functions, variables, constants, imports, structs, enums, traits, impls).

use sumer_ast::{
    Attribute, ConstantDecl, Declaration, EnumDecl, EnumVariant, FieldDecl, FunctionDecl,
    FunctionSignature, GenericParam, GenericParams, ImplDecl, ImportDecl, ModulePath, Parameter,
    StructDecl, StructMember, TraitDecl, TraitMember, TypeBound, VariableDecl, VariantData,
    Visibility,
};
use sumer_lexer::TokenKind;
use sumer_span::Span;

use crate::attribute::parse_attributes;
use crate::cursor::TokenCursor;
use crate::error::{ParseError, ParseErrorKind, ParseResult};
use crate::expression::parse_expression;
use crate::statement::parse_block;
use crate::types::parse_type;

/// Parses a top-level declaration with preceding attributes.
pub fn parse_declaration(
    cursor: &mut TokenCursor<'_>,
    attributes: Vec<Attribute>,
) -> ParseResult<Declaration> {
    let attr_start = attributes.first().map(|a| a.span);
    let mut visibility = Visibility::Private;
    let mut is_async = false;
    let mut mod_start: Option<Span> = None;

    loop {
        if cursor.match_token(&TokenKind::Pub) {
            visibility = Visibility::Public;
            if mod_start.is_none() {
                mod_start = Some(cursor.previous().span());
            }
        } else if cursor.match_token(&TokenKind::Async) {
            is_async = true;
            if mod_start.is_none() {
                mod_start = Some(cursor.previous().span());
            }
        } else {
            break;
        }
    }

    let start_span = attr_start.or(mod_start);

    match cursor.current().kind() {
        TokenKind::Fn => {
            let func = parse_function(cursor, visibility, is_async, attributes, start_span)?;
            Ok(Declaration::Function(func))
        }
        TokenKind::Struct => {
            if is_async {
                return Err(ParseError::new(
                    ParseErrorKind::InvalidDeclaration("struct cannot be declared async".into()),
                    "struct cannot be declared async",
                    cursor.current().span(),
                ));
            }
            let s = parse_struct_decl(cursor, visibility, attributes, start_span)?;
            Ok(Declaration::Struct(s))
        }
        TokenKind::Enum => {
            if is_async {
                return Err(ParseError::new(
                    ParseErrorKind::InvalidDeclaration("enum cannot be declared async".into()),
                    "enum cannot be declared async",
                    cursor.current().span(),
                ));
            }
            let e = parse_enum_decl(cursor, visibility, attributes, start_span)?;
            Ok(Declaration::Enum(e))
        }
        TokenKind::Trait => {
            if is_async {
                return Err(ParseError::new(
                    ParseErrorKind::InvalidDeclaration("trait cannot be declared async".into()),
                    "trait cannot be declared async",
                    cursor.current().span(),
                ));
            }
            let t = parse_trait_decl(cursor, visibility, attributes, start_span)?;
            Ok(Declaration::Trait(t))
        }
        TokenKind::Impl => {
            if is_async {
                return Err(ParseError::new(
                    ParseErrorKind::InvalidDeclaration("impl cannot be declared async".into()),
                    "impl cannot be declared async",
                    cursor.current().span(),
                ));
            }
            let i = parse_impl_decl(cursor, attributes, start_span)?;
            Ok(Declaration::Impl(i))
        }
        TokenKind::Let | TokenKind::Var => {
            if is_async {
                return Err(ParseError::new(
                    ParseErrorKind::InvalidDeclaration("variables cannot be declared async".into()),
                    "variables cannot be declared async",
                    cursor.current().span(),
                ));
            }
            let var_decl = parse_variable_decl(cursor, visibility, start_span)?;
            Ok(Declaration::Variable(var_decl))
        }
        TokenKind::Const => {
            if is_async {
                return Err(ParseError::new(
                    ParseErrorKind::InvalidDeclaration("constants cannot be declared async".into()),
                    "constants cannot be declared async",
                    cursor.current().span(),
                ));
            }
            let const_decl = parse_constant_decl(cursor, visibility, attributes, start_span)?;
            Ok(Declaration::Constant(const_decl))
        }
        TokenKind::Import => {
            if is_async {
                return Err(ParseError::new(
                    ParseErrorKind::InvalidDeclaration("imports cannot be declared async".into()),
                    "imports cannot be declared async",
                    cursor.current().span(),
                ));
            }
            let import_decl = parse_import_decl(cursor, visibility, start_span)?;
            Ok(Declaration::Import(import_decl))
        }
        _ => Err(ParseError::unexpected_token(
            "declaration",
            cursor.current(),
        )),
    }
}

/// Parses generic parameters with optional bounds: `<T: Clone + Debug, U>`.
pub fn parse_generic_params(
    cursor: &mut TokenCursor<'_>,
    fallback_span: Span,
) -> ParseResult<GenericParams> {
    if !cursor.match_token(&TokenKind::Less) {
        return Ok(GenericParams::new(Vec::new(), fallback_span));
    }

    let start = cursor.previous().span();
    let mut params = Vec::new();

    if !cursor.check(&TokenKind::Greater) {
        params.push(parse_generic_param(cursor)?);
        while cursor.match_token(&TokenKind::Comma) {
            if cursor.check(&TokenKind::Greater) {
                break;
            }
            params.push(parse_generic_param(cursor)?);
        }
    }

    let r_ang = cursor.expect(&TokenKind::Greater)?;
    let span = start.join(r_ang.span()).unwrap_or(start);
    Ok(GenericParams::new(params, span))
}

fn parse_generic_param(cursor: &mut TokenCursor<'_>) -> ParseResult<GenericParam> {
    let name = cursor.parse_identifier()?;
    let start = name.span();
    let mut bounds = Vec::new();
    let mut end = start;

    if cursor.match_token(&TokenKind::Colon) {
        let b_name = cursor.parse_identifier()?;
        let b_span = b_name.span();
        end = b_span;
        bounds.push(TypeBound::new(b_name, b_span));

        while cursor.match_token(&TokenKind::Plus) {
            let b_name = cursor.parse_identifier()?;
            let b_span = b_name.span();
            end = b_span;
            bounds.push(TypeBound::new(b_name, b_span));
        }
    }

    let span = start.join(end).unwrap_or(start);
    Ok(GenericParam::new(name, bounds, span))
}

/// Parses parameter list inside `( ... )`.
fn parse_parameter_list(cursor: &mut TokenCursor<'_>) -> ParseResult<Vec<Parameter>> {
    cursor.expect(&TokenKind::LParen)?;
    let mut parameters = Vec::new();

    if !cursor.check(&TokenKind::RParen) {
        parameters.push(parse_parameter(cursor)?);
        while cursor.match_token(&TokenKind::Comma) {
            if cursor.check(&TokenKind::RParen) {
                break;
            }
            parameters.push(parse_parameter(cursor)?);
        }
    }

    cursor.expect(&TokenKind::RParen)?;
    Ok(parameters)
}

/// Parses a function declaration (`[pub] [async] fn name(...) [-> type] { ... }`).
pub fn parse_function(
    cursor: &mut TokenCursor<'_>,
    visibility: Visibility,
    is_async: bool,
    attributes: Vec<Attribute>,
    start_span: Option<Span>,
) -> ParseResult<FunctionDecl> {
    let fn_tok = cursor.expect(&TokenKind::Fn)?;
    let start = start_span.unwrap_or(fn_tok.span());

    let name = cursor.parse_identifier()?;
    let generics = parse_generic_params(cursor, name.span())?;
    let parameters = parse_parameter_list(cursor)?;

    let return_type = if cursor.match_token(&TokenKind::Arrow) {
        Some(parse_type(cursor)?)
    } else {
        None
    };

    let body = parse_block(cursor)?;
    let span = start.join(body.span).unwrap_or(start);

    Ok(FunctionDecl::new(
        name,
        visibility,
        is_async,
        generics,
        parameters,
        return_type,
        body,
        attributes,
        span,
    ))
}

/// Parses an individual formal parameter (`name: Type [= default]`).
pub fn parse_parameter(cursor: &mut TokenCursor<'_>) -> ParseResult<Parameter> {
    let name = cursor.parse_identifier()?;
    let start = name.span();

    cursor.expect(&TokenKind::Colon)?;
    let param_type = parse_type(cursor)?;

    let default_value = if cursor.match_token(&TokenKind::Equal) {
        Some(parse_expression(cursor)?)
    } else {
        None
    };

    let end = default_value
        .as_ref()
        .map(|e| e.span)
        .unwrap_or(param_type.span);
    let span = start.join(end).unwrap_or(start);

    Ok(Parameter::new(name, param_type, default_value, span))
}

/// Parses a variable binding declaration (`let` or `var`).
pub fn parse_variable_decl(
    cursor: &mut TokenCursor<'_>,
    visibility: Visibility,
    start_span: Option<Span>,
) -> ParseResult<VariableDecl> {
    let (is_mutable, kw_span) = if cursor.match_token(&TokenKind::Var) {
        (true, cursor.previous().span())
    } else if cursor.match_token(&TokenKind::Let) {
        (false, cursor.previous().span())
    } else {
        return Err(ParseError::unexpected_token("let or var", cursor.current()));
    };

    let start = start_span.unwrap_or(kw_span);
    let name = cursor.parse_identifier()?;

    let explicit_type = if cursor.match_token(&TokenKind::Colon) {
        Some(parse_type(cursor)?)
    } else {
        None
    };

    cursor.expect(&TokenKind::Equal)?;
    let initializer = parse_expression(cursor)?;
    cursor.consume_semicolon_if_present();

    let span = start.join(initializer.span).unwrap_or(start);

    Ok(VariableDecl::new(
        name,
        is_mutable,
        explicit_type,
        initializer,
        visibility,
        span,
    ))
}

/// Parses a constant declaration (`const MAX [: Type] = expr`).
pub fn parse_constant_decl(
    cursor: &mut TokenCursor<'_>,
    visibility: Visibility,
    attributes: Vec<Attribute>,
    start_span: Option<Span>,
) -> ParseResult<ConstantDecl> {
    let const_tok = cursor.expect(&TokenKind::Const)?;
    let start = start_span.unwrap_or(const_tok.span());

    let name = cursor.parse_identifier()?;

    let explicit_type = if cursor.match_token(&TokenKind::Colon) {
        Some(parse_type(cursor)?)
    } else {
        None
    };

    cursor.expect(&TokenKind::Equal)?;
    let value = parse_expression(cursor)?;
    cursor.consume_semicolon_if_present();

    let span = start.join(value.span).unwrap_or(start);

    Ok(ConstantDecl::new(
        name,
        explicit_type,
        value,
        visibility,
        attributes,
        span,
    ))
}

/// Parses an import declaration (`import user.User` or `import foo.bar.Baz`).
pub fn parse_import_decl(
    cursor: &mut TokenCursor<'_>,
    visibility: Visibility,
    start_span: Option<Span>,
) -> ParseResult<ImportDecl> {
    let import_tok = cursor.expect(&TokenKind::Import)?;
    let start = start_span.unwrap_or(import_tok.span());

    let first = cursor.parse_identifier()?;
    let mut segments = vec![first];

    while cursor.match_token(&TokenKind::Dot) {
        segments.push(cursor.parse_identifier()?);
    }

    cursor.consume_semicolon_if_present();

    let first_span = segments[0].span();
    let last_span = segments.last().map(|s| s.span()).unwrap_or(first_span);
    let path_span = first_span.join(last_span).unwrap_or(first_span);
    let path = ModulePath::new(segments, path_span);

    let span = start.join(last_span).unwrap_or(start);

    Ok(ImportDecl::new(path, visibility, span))
}

/// Parses a struct declaration (`struct User<T> { ... }`).
pub fn parse_struct_decl(
    cursor: &mut TokenCursor<'_>,
    visibility: Visibility,
    attributes: Vec<Attribute>,
    start_span: Option<Span>,
) -> ParseResult<StructDecl> {
    let struct_tok = cursor.expect(&TokenKind::Struct)?;
    let start = start_span.unwrap_or(struct_tok.span());

    let name = cursor.parse_identifier()?;
    let generics = parse_generic_params(cursor, name.span())?;

    cursor.expect(&TokenKind::LBrace)?;
    let mut members = Vec::new();

    while !cursor.check(&TokenKind::RBrace) && !cursor.is_at_end() {
        let member_attrs = parse_attributes(cursor)?;
        let member_vis = if cursor.match_token(&TokenKind::Pub) {
            Visibility::Public
        } else {
            Visibility::Private
        };

        if cursor.check(&TokenKind::Fn) || cursor.check(&TokenKind::Async) {
            let is_async = cursor.match_token(&TokenKind::Async);
            let func = parse_function(cursor, member_vis, is_async, member_attrs, None)?;
            members.push(StructMember::Method(func));
        } else {
            let field_name = cursor.parse_identifier()?;
            cursor.expect(&TokenKind::Colon)?;
            let field_type = parse_type(cursor)?;
            let default_value = if cursor.match_token(&TokenKind::Equal) {
                Some(parse_expression(cursor)?)
            } else {
                None
            };
            cursor.consume_semicolon_if_present();
            cursor.match_token(&TokenKind::Comma);
            let field_span = field_name
                .span()
                .join(field_type.span)
                .unwrap_or(field_name.span());
            members.push(StructMember::Field(FieldDecl::new(
                field_name,
                member_vis,
                field_type,
                default_value,
                member_attrs,
                field_span,
            )));
        }
    }

    let rbrace = cursor.expect(&TokenKind::RBrace)?;
    let span = start.join(rbrace.span()).unwrap_or(start);

    Ok(StructDecl::new(
        name, visibility, generics, members, attributes, span,
    ))
}

/// Parses an enum declaration (`enum Color { ... }`).
pub fn parse_enum_decl(
    cursor: &mut TokenCursor<'_>,
    visibility: Visibility,
    attributes: Vec<Attribute>,
    start_span: Option<Span>,
) -> ParseResult<EnumDecl> {
    let enum_tok = cursor.expect(&TokenKind::Enum)?;
    let start = start_span.unwrap_or(enum_tok.span());

    let name = cursor.parse_identifier()?;
    let generics = parse_generic_params(cursor, name.span())?;

    cursor.expect(&TokenKind::LBrace)?;
    let mut variants = Vec::new();

    while !cursor.check(&TokenKind::RBrace) && !cursor.is_at_end() {
        let var_name = cursor.parse_identifier()?;
        let var_start = var_name.span();

        let (data, var_span) = if cursor.match_token(&TokenKind::LParen) {
            let mut types = Vec::new();
            if !cursor.check(&TokenKind::RParen) {
                types.push(parse_type(cursor)?);
                while cursor.match_token(&TokenKind::Comma) {
                    if cursor.check(&TokenKind::RParen) {
                        break;
                    }
                    types.push(parse_type(cursor)?);
                }
            }
            let rparen = cursor.expect(&TokenKind::RParen)?;
            let span = var_start.join(rparen.span()).unwrap_or(var_start);
            (VariantData::Tuple(types), span)
        } else if cursor.match_token(&TokenKind::LBrace) {
            let mut fields = Vec::new();
            while !cursor.check(&TokenKind::RBrace) && !cursor.is_at_end() {
                let f_attrs = parse_attributes(cursor)?;
                let f_vis = if cursor.match_token(&TokenKind::Pub) {
                    Visibility::Public
                } else {
                    Visibility::Private
                };
                let f_name = cursor.parse_identifier()?;
                cursor.expect(&TokenKind::Colon)?;
                let f_type = parse_type(cursor)?;
                let default_val = if cursor.match_token(&TokenKind::Equal) {
                    Some(parse_expression(cursor)?)
                } else {
                    None
                };
                cursor.consume_semicolon_if_present();
                cursor.match_token(&TokenKind::Comma);
                let f_span = f_name.span().join(f_type.span).unwrap_or(f_name.span());
                fields.push(FieldDecl::new(
                    f_name,
                    f_vis,
                    f_type,
                    default_val,
                    f_attrs,
                    f_span,
                ));
            }
            let rbrace = cursor.expect(&TokenKind::RBrace)?;
            let span = var_start.join(rbrace.span()).unwrap_or(var_start);
            (VariantData::Struct(fields), span)
        } else {
            (VariantData::Unit, var_start)
        };

        variants.push(EnumVariant::new(var_name, data, var_span));
        cursor.match_token(&TokenKind::Comma);
        cursor.consume_semicolon_if_present();
    }

    let rbrace = cursor.expect(&TokenKind::RBrace)?;
    let span = start.join(rbrace.span()).unwrap_or(start);

    Ok(EnumDecl::new(
        name, visibility, generics, variants, attributes, span,
    ))
}

/// Parses a trait declaration (`trait Printable<T>: Debug + Clone { ... }`).
pub fn parse_trait_decl(
    cursor: &mut TokenCursor<'_>,
    visibility: Visibility,
    attributes: Vec<Attribute>,
    start_span: Option<Span>,
) -> ParseResult<TraitDecl> {
    let trait_tok = cursor.expect(&TokenKind::Trait)?;
    let start = start_span.unwrap_or(trait_tok.span());

    let name = cursor.parse_identifier()?;
    let generics = parse_generic_params(cursor, name.span())?;

    // Supertrait bounds: `: Debug + Clone`
    let mut bounds = Vec::new();
    if cursor.match_token(&TokenKind::Colon) {
        let b_name = cursor.parse_identifier()?;
        let b_span = b_name.span();
        bounds.push(TypeBound::new(b_name, b_span));
        while cursor.match_token(&TokenKind::Plus) {
            let b_name = cursor.parse_identifier()?;
            let b_span = b_name.span();
            bounds.push(TypeBound::new(b_name, b_span));
        }
    }

    cursor.expect(&TokenKind::LBrace)?;
    let mut members = Vec::new();

    while !cursor.check(&TokenKind::RBrace) && !cursor.is_at_end() {
        let m_attrs = parse_attributes(cursor)?;
        let m_async = cursor.match_token(&TokenKind::Async);
        cursor.expect(&TokenKind::Fn)?;
        let m_name = cursor.parse_identifier()?;
        let m_start = m_name.span();
        let m_generics = parse_generic_params(cursor, m_start)?;
        let parameters = parse_parameter_list(cursor)?;
        let return_type = if cursor.match_token(&TokenKind::Arrow) {
            Some(parse_type(cursor)?)
        } else {
            None
        };

        if cursor.check(&TokenKind::LBrace) {
            let body = parse_block(cursor)?;
            let span = m_start.join(body.span).unwrap_or(m_start);
            let func = FunctionDecl::new(
                m_name,
                Visibility::Public,
                m_async,
                m_generics,
                parameters,
                return_type,
                body,
                m_attrs,
                span,
            );
            members.push(TraitMember::Function(func));
        } else {
            cursor.consume_semicolon_if_present();
            let end = return_type
                .as_ref()
                .map(|t| t.span)
                .or_else(|| parameters.last().map(|p| p.span))
                .unwrap_or(m_start);
            let span = m_start.join(end).unwrap_or(m_start);
            let sig =
                FunctionSignature::new(m_name, m_async, m_generics, parameters, return_type, span);
            members.push(TraitMember::FunctionSignature(sig));
        }
    }

    let rbrace = cursor.expect(&TokenKind::RBrace)?;
    let span = start.join(rbrace.span()).unwrap_or(start);

    Ok(TraitDecl::new(
        name, visibility, generics, bounds, members, attributes, span,
    ))
}

/// Parses an impl declaration (`impl User { ... }` or `impl Printable for User { ... }`).
pub fn parse_impl_decl(
    cursor: &mut TokenCursor<'_>,
    attributes: Vec<Attribute>,
    start_span: Option<Span>,
) -> ParseResult<ImplDecl> {
    let impl_tok = cursor.expect(&TokenKind::Impl)?;
    let start = start_span.unwrap_or(impl_tok.span());

    let generics = parse_generic_params(cursor, start)?;
    let first_type = parse_type(cursor)?;

    let (trait_type, target_type) = if cursor.match_token(&TokenKind::For) {
        let target = parse_type(cursor)?;
        (Some(first_type), target)
    } else {
        (None, first_type)
    };

    cursor.expect(&TokenKind::LBrace)?;
    let mut members = Vec::new();

    while !cursor.check(&TokenKind::RBrace) && !cursor.is_at_end() {
        let m_attrs = parse_attributes(cursor)?;
        let m_vis = if cursor.match_token(&TokenKind::Pub) {
            Visibility::Public
        } else {
            Visibility::Private
        };
        let m_async = cursor.match_token(&TokenKind::Async);
        let func = parse_function(cursor, m_vis, m_async, m_attrs, None)?;
        members.push(func);
    }

    let rbrace = cursor.expect(&TokenKind::RBrace)?;
    let span = start.join(rbrace.span()).unwrap_or(start);

    Ok(ImplDecl::new(
        generics,
        trait_type,
        target_type,
        members,
        attributes,
        span,
    ))
}
