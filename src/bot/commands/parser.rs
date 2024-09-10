// Spaghetti code 😬

use std::{ffi::OsString, iter::Peekable, mem::take, os::unix::ffi::OsStringExt};

pub struct MsgParser<'a> {
    prefix: &'a str,
    /// Line iterator
    data: Box<dyn Iterator<Item = &'a str> + 'a>,
}

impl<'a> MsgParser<'a> {
    pub fn new(prefix: &'a str, msg: &'a str) -> Self {
        Self {
            prefix,
            data: Box::new(msg.lines()),
        }
    }

    pub fn parse(&mut self) -> Result<Vec<ShellArgs>, ParseError> {
        let mut shell_commands = vec![];
        let mut parser = None;

        for line in &mut *self.data {
            let (mut local_parser, line_iter) = match parser {
                Some(parser) => (parser, line.chars()),
                None => {
                    if !line.starts_with(self.prefix) {
                        continue;
                    }
                    (ParseCtx::default(), line[self.prefix.len()..].chars())
                }
            };

            if let Some(_) = local_parser.push_chars(&mut line_iter.peekable())? {
                return Err(ParseError::IllegalRootUnnest);
            }

            match local_parser.close()? {
                Some(args) => {
                    shell_commands.push(args);
                    parser = None;
                }
                None => parser = Some(local_parser),
            };
        }

        if parser.is_some() {
            return Err(ParseError::UnfinishedLastCommand);
        }

        Ok(shell_commands)
    }
}

pub trait Environ {
    // TODO: return Option<&OsStr> as soon as we remove the dummy impl
    //  ( the reference should be valid for all Environ's lifetime, and this
    //  one would keep track of those internally ig )
    fn get(&mut self, key: &str) -> Option<OsString>;
}

pub trait Executer {
    fn execute(&mut self, args: Vec<OsString>, env: &mut impl Environ) -> OsString;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExecuteError {
    NoSuchEnv,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct ShellArgs(Vec<Vec<ShellArg>>);

impl ShellArgs {
    pub fn resolve(
        self,
        environ: &mut impl Environ,
        executer: &mut impl Executer,
    ) -> Result<OsString, ExecuteError> {
        let arg_list: Vec<_> = self
            .0
            .into_iter()
            .map(|arg| -> Result<OsString, ExecuteError> {
                let mut arg_string = OsString::new();
                arg.into_iter()
                    .try_for_each(|component| -> Result<(), ExecuteError> {
                        match component {
                            ShellArg::Byte(byte) => arg_string.push(OsString::from_vec(vec![byte])),
                            ShellArg::Char(ch) => arg_string.push(ch.to_string()),
                            ShellArg::RawString(rstring) => arg_string.push(rstring),
                            ShellArg::String(string) => arg_string.push(string),
                            ShellArg::EnvVar(env_ref) => arg_string
                                .push(environ.get(&env_ref).ok_or(ExecuteError::NoSuchEnv)?),
                            ShellArg::Subshell(args) => {
                                arg_string.push(args.resolve(environ, executer)?)
                            }
                        };
                        Ok(())
                    })?;
                Ok(arg_string)
            })
            .try_collect()?;

        Ok(executer.execute(arg_list, environ))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ShellArg {
    Byte(u8),
    Char(char),
    RawString(OsString),
    String(String),
    EnvVar(String),
    Subshell(ShellArgs),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseAction {
    Separator,
    Push(ShellArg),
    Unnest,
    Nest(ParseCtxType),
    /// Take next byte and try to call a multibyte sequence fn
    EscapeSequence,
}

#[derive(Debug)]
pub enum ParseError {
    IllegalRootUnnest,
    InvalidEscapeSequence,
    UnexpectedCloser(char),
    UnfinishedLastCommand,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseCtxType {
    /// Bool represents root level (unclosable)
    Normal(bool),
    /// Bool represents strict (same as single quote in bash)
    Quote(bool),
}
impl Default for ParseCtxType {
    fn default() -> Self {
        Self::Normal(true)
    }
}
impl Default for &ParseCtxType {
    fn default() -> Self {
        &ParseCtxType::Normal(true)
    }
}

#[derive(Default)]
pub struct ParseCtx {
    pub typ: ParseCtxType,
    escape: String,
    arg: Vec<ShellArg>,
    args: ShellArgs,
    nesting: Box<Option<Self>>,
    just_separated: bool,
}

impl ParseCtxType {
    const ESCAPABLE_CHARS: &'static [char] = &['\\', ' ', '(', ')', '{', '}'];
    const ESCAPABLE_BYTES: &'static [(char, char)] =
        &[('n', '\n'), ('r', '\r'), ('t', '\t'), ('0', '\0')];

    pub fn closer_token(&self) -> char {
        match self {
            Self::Normal(true) => '\n',
            Self::Normal(false) => ')',
            Self::Quote(true) => '\'',
            Self::Quote(false) => '"',
        }
    }

    pub fn token(&self, token: char) -> Result<ParseAction, ParseError> {
        match (self, token) {
            (Self::Normal(_), ' ') => Ok(ParseAction::Separator),
            (Self::Normal(_), '\'' | '"') => Ok(ParseAction::Nest(Self::Quote(token == '\''))),
            (Self::Quote(_), ' ') => Ok(ParseAction::Push(ShellArg::Char(token))),
            (_, '\\' | '$') => Ok(ParseAction::EscapeSequence),
            (_, '\n' | '\'' | '"' | ')') => {
                if token == self.closer_token() {
                    Ok(ParseAction::Unnest)
                } else {
                    match self {
                        Self::Normal(_) => Err(ParseError::UnexpectedCloser(token)),
                        _ => Ok(ParseAction::Push(ShellArg::Char(token))),
                    }
                }
            }
            _ => Ok(ParseAction::Push(ShellArg::Char(token))),
        }
    }

    // bool represents if the new character should be re-enqueed
    pub fn escape(&self, escape: &str, new_char: char) -> Result<(ParseAction, bool), ParseError> {
        let mut chars_iter = escape.chars();
        let Some(escape_type) = chars_iter.next() else {
            return Err(ParseError::InvalidEscapeSequence);
        };

        match escape_type {
            '\\' => {
                let Some(discriminator) = chars_iter.next() else {
                    if Self::ESCAPABLE_CHARS.iter().any(|&c| c == new_char) {
                        return Ok((ParseAction::Push(ShellArg::Char(new_char)), false));
                    }

                    if let Some((_, byte)) =
                        Self::ESCAPABLE_BYTES.iter().find(|&(c, _)| *c == new_char)
                    {
                        return Ok((ParseAction::Push(ShellArg::Char(*byte)), false));
                    }

                    return Ok((ParseAction::EscapeSequence, false));
                };

                match discriminator {
                    'x' => {
                        if escape.len() > 3 {
                            return Err(ParseError::InvalidEscapeSequence);
                        }

                        let Some(chars) = chars_iter.next() else {
                            return Ok((ParseAction::EscapeSequence, false));
                        };

                        let byte_val = u8::from_str_radix(&format!("{chars}{new_char}"), 16)
                            .map_err(|_| ParseError::InvalidEscapeSequence)?;
                        return Ok((ParseAction::Push(ShellArg::Byte(byte_val)), false));
                    }
                    'u' => {
                        if escape.len() > 11 {
                            return Err(ParseError::InvalidEscapeSequence);
                        }

                        if new_char != '}' {
                            return Ok((ParseAction::EscapeSequence, false));
                        }

                        let Some('{') = chars_iter.next() else {
                            return Err(ParseError::InvalidEscapeSequence);
                        };

                        let value: String = chars_iter.collect();
                        let unicode_point = u32::from_str_radix(&value, 16)
                            .map_err(|_| ParseError::InvalidEscapeSequence)?;
                        let unicode_char = char::from_u32(unicode_point)
                            .ok_or(ParseError::InvalidEscapeSequence)?;

                        Ok((ParseAction::Push(ShellArg::Char(unicode_char)), false))
                    }
                    _ => Err(ParseError::InvalidEscapeSequence),
                }
            }
            '$' => {
                let Some(discriminator) = chars_iter.next() else {
                    return Ok((ParseAction::EscapeSequence, false));
                };

                if discriminator == '(' {
                    Ok((ParseAction::Nest(Self::Normal(false)), true))
                } else if discriminator == '{' {
                    if new_char == '}' {
                        Ok((
                            ParseAction::Push(ShellArg::EnvVar(chars_iter.collect())),
                            false,
                        ))
                    } else {
                        Ok((ParseAction::EscapeSequence, false))
                    }
                } else if discriminator.is_alphanumeric() || discriminator == '_' {
                    if !new_char.is_alphanumeric() && new_char != '_' {
                        let mut env_name = discriminator.to_string();
                        chars_iter.for_each(|c| env_name.push(c));

                        Ok((ParseAction::Push(ShellArg::EnvVar(env_name)), true))
                    } else {
                        Ok((ParseAction::EscapeSequence, false))
                    }
                } else {
                    Err(ParseError::InvalidEscapeSequence)
                }
            }
            _ => Err(ParseError::InvalidEscapeSequence),
        }
    }
}

impl ParseCtx {
    pub fn new(typ: ParseCtxType) -> Self {
        let mut myself = Self::default();
        myself.typ = typ;
        myself
    }

    pub fn from_chars(iter: &mut impl Iterator<Item = char>) -> Result<ShellArgs, ParseError> {
        let mut new_ctx = Self::default();
        if let Some(_) = new_ctx.push_chars(&mut iter.peekable())? {
            return Err(ParseError::IllegalRootUnnest);
        }
        new_ctx.forced_close()
    }

    pub fn push_chars(
        &mut self,
        iter: &mut Peekable<impl Iterator<Item = char>>,
    ) -> Result<Option<ShellArgs>, ParseError> {
        while let Some(ch) = iter.next() {
            if let Some(args) = self.push_char(ch)? {
                if iter.peek().is_some() {
                    return Err(ParseError::IllegalRootUnnest);
                }
                return Ok(Some(args));
            };
        }
        Ok(None)
    }

    pub fn push_char(&mut self, ch: char) -> Result<Option<ShellArgs>, ParseError> {
        if let Some(ref mut nesting) = *(self.nesting) {
            if let Some(args) = nesting.push_char(ch)? {
                if nesting.typ == ParseCtxType::Normal(false) {
                    self.arg.push(ShellArg::Subshell(args));
                } else {
                    let mut flatten = args.0.into_iter().flatten().collect();
                    self.arg.append(&mut flatten);
                }
                *self.nesting = None;
            };
            return Ok(None);
        };

        let (act, requeue) = if self.escape.is_empty() {
            (self.typ.token(ch)?, false)
        } else {
            let (act, requeue) = self.typ.escape(&self.escape, ch)?;
            if act != ParseAction::EscapeSequence {
                self.escape = String::new();
            };
            (act, requeue)
        };

        let just_separated_binding = act == ParseAction::Separator;
        match act {
            ParseAction::Nest(ctx_typ) => *self.nesting = Some(Self::new(ctx_typ)),
            ParseAction::Unnest => match *self.nesting {
                Some(_) => return Err(ParseError::IllegalRootUnnest),
                None => {
                    return Ok(Some(self.forced_close()?));
                }
            },
            ParseAction::EscapeSequence => self.escape.push(ch),
            ParseAction::Push(token) => self.arg.push(token),
            ParseAction::Separator => {
                if !self.just_separated {
                    self.args.0.push(take(&mut self.arg));
                }
            }
        };
        self.just_separated = just_separated_binding;

        if requeue {
            self.push_char(ch)
        } else {
            Ok(None)
        }
    }

    pub fn forced_close(&mut self) -> Result<ShellArgs, ParseError> {
        self.close()?.ok_or(ParseError::InvalidEscapeSequence)
    }

    pub fn close(&mut self) -> Result<Option<ShellArgs>, ParseError> {
        if self.nesting.is_some() {
            return Err(ParseError::IllegalRootUnnest);
        }

        match self.escape.as_ref() {
            "\\" => Ok(None),
            "" => {
                self.args.0.push(take(&mut self.arg));
                Ok(Some(take(&mut self.args)))
            }
            _ => {
                //  This allows to type env variables starting with '{' as long
                // as they're at closure, not expected behavior but is harmless
                if self.escape.starts_with('$') {
                    self.arg.push(ShellArg::EnvVar(self.escape[1..].to_owned()));
                    self.escape = String::new();
                    return self.close();
                }
                Err(ParseError::InvalidEscapeSequence)
            }
        }
    }
}
