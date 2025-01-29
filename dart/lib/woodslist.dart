// Core data structures

abstract class Wood {
  final int line;
  final int column;

  Wood(this.line, this.column);

  static Wood from(Object v) {
    if (v is List) {
      return Branch(v: v.map((o) => Wood.from(o)).toList(), line: 0, column: 0);
    } else if (v is String) {
      return Leaf(v: v, line: 0, column: 0);
    } else {
      throw WoodConversionError();
    }
  }

  String initialStr();
  bool get isLeaf;
  bool get isBranch;
  @override
  String toString() => woodToString(this);
}

class Branch extends Wood {
  final List<Wood> v;

  Branch({
    required this.v,
    required int line,
    required int column,
  }) : super(line, column);

  @override
  bool get isLeaf => false;

  @override
  bool get isBranch => true;

  @override
  String initialStr() {
    if (v.isEmpty) return '';
    return v.first.initialStr();
  }

  @override
  bool operator ==(Object other) =>
      other is Branch &&
      other.v.length == v.length &&
      List.generate(v.length, (i) => v[i] == other.v[i]).every((x) => x);

  @override
  int get hashCode => Object.hashAll(v);
}

class Leaf extends Wood {
  final String v;

  Leaf({
    required this.v,
    required int line,
    required int column,
  }) : super(line, column);

  @override
  bool get isLeaf => true;

  @override
  bool get isBranch => false;

  @override
  String initialStr() => v;

  @override
  bool operator ==(Object other) => other is Leaf && other.v == v;

  @override
  int get hashCode => v.hashCode;
}

Wood branch(List<Wood> v) => Branch(v: v, line: 0, column: 0);
Wood leaf(String v) => Leaf(v: v, line: 0, column: 0);

class WoodConversionError extends TypeError {
  WoodConversionError();

  @override
  String toString() =>
      'WoodConversionError: attempt to convert an Object into a Wood that was neither a List nor a String';
}

class WoodError implements Exception {
  final int line;
  final int column;
  final String message;
  final Exception? cause;

  WoodError(
    this.message, {
    this.line = -1,
    this.column = -1,
    this.cause,
  });

  @override
  String toString() => 'WoodError: $message (at line: $line, column: $column)';
}

enum _ParseResult {
  foundEof,
  foundParen,
}

typedef Rune = int;
// necessitated by dart not having good enough const evaluation

const Rune NEWLINE = 10; // = '\n'.runes.first;
const Rune CARRIAGE_RETURN = 13; // = '\r'.runes.first;
const Rune TAB = 9; // = '\t'.runes.first;
const Rune SNOWMAN = 9731; // = '☃'.runes.first;
const Rune QUOTE = 34; // = '"'.runes.first;
const Rune ESCAPE_SLASH = 92; // = '\\'.runes.first;
const Rune OPEN_PAREN = 40;
const Rune CLOSE_PAREN = 41;
const Rune SPACE = 32;
const Rune LETTER_N = 110;
const Rune LETTER_R = 114;
const Rune LETTER_T = 116;
const Rune LETTER_H = 104;

// Parser implementation
class WoodParser {
  final String input;
  final RuneIterator _runes;
  int _line = 1;
  int _column = 1;

  WoodParser(this.input) : _runes = input.runes.iterator {
    _runes.moveNext();
  }

  Rune? _translateWindowsReturn(Rune? v) => v != null
      ? v == CARRIAGE_RETURN
          ? NEWLINE
          : v
      : null;

  Rune? _peek() {
    Rune ret = _runes.current;
    if (ret != -1) {
      return _translateWindowsReturn(ret);
    } else {
      return null;
    }
  }

  Rune? _next() {
    Rune? ret = _peek();
    _runes.moveNext();

    if (ret == '\n') {
      _line++;
      _column = 1;
    } else if (ret == '\r') {
      if (_runes.current == '\n') {
        _runes.moveNext();
      }
      _line++;
      _column = 1;
    } else {
      _column++;
    }
    return ret;
  }

  Rune _readEscapedChar() {
    final next = _next();
    if (next == null) {
      throw WoodError(
        'Escape slash must be followed by a valid escape character code',
        line: _line,
        column: _column,
      );
    }

    switch (next) {
      case LETTER_N:
        return NEWLINE;
      case LETTER_R:
        return CARRIAGE_RETURN;
      case LETTER_T:
        return TAB;
      case LETTER_H:
        return SNOWMAN;
      case QUOTE:
        return QUOTE;
      case ESCAPE_SLASH:
        return ESCAPE_SLASH;
      default:
        throw WoodError(
          'Invalid escape sequence: \\$next',
          line: _line,
          column: _column,
        );
    }
  }

  /// returns a Branch containing every item in the file
  Wood parseMultiple() {
    final root = Branch(v: [], line: 1, column: 1);
    final result = _parseInternal(root);

    if (result == _ParseResult.foundParen) {
      throw WoodError(
        'Unmatched closing parenthesis',
        line: _line,
        column: _column,
      );
    }

    return root;
  }

  Wood parse() {
    final root = parseMultiple();
    if ((root as Branch).v.length == 1) {
      return root.v.first;
    }
    return root;
  }

  _ParseResult _parseInternal(Branch into) {
    while (true) {
      final char = _next();
      if (char == null) return _ParseResult.foundEof;

      switch (char) {
        case OPEN_PAREN:
          final newBranch = Branch(
            v: [],
            line: _line,
            column: _column,
          );
          into.v.add(newBranch);

          final result = _parseInternal(newBranch);
          if (result == _ParseResult.foundEof) {
            throw WoodError(
              'Unmatched opening parenthesis',
              line: newBranch.line,
              column: newBranch.column,
            );
          }
          break;

        case CLOSE_PAREN:
          return _ParseResult.foundParen;

        case SPACE:
        case TAB:
        case NEWLINE:
          break;

        case QUOTE:
          final int l = _line;
          final int c = _column;

          // Skip initial newline if present
          if (_peek() == NEWLINE) {
            _next();
          }

          final buffer = StringBuffer();
          while (true) {
            final nc = _next();
            if (nc == null) {
              throw WoodError(
                'Unclosed string',
                line: l,
                column: c,
              );
            }
            if (nc == QUOTE) break;
            if (nc == ESCAPE_SLASH) {
              buffer.writeCharCode(_readEscapedChar());
            } else {
              buffer.writeCharCode(nc);
            }
          }

          into.v.add(Leaf(
            v: buffer.toString(),
            line: l,
            column: c,
          ));
          break;

        default:
          final l = _line;
          final c = _column;

          final buffer = StringBuffer(String.fromCharCode(char));
          while (true) {
            final next = _peek();
            if (next == null ||
                next == SPACE ||
                next == TAB ||
                next == NEWLINE ||
                next == QUOTE ||
                next == OPEN_PAREN ||
                next == CLOSE_PAREN) {
              break;
            }

            final nc = _next()!;
            if (nc == ESCAPE_SLASH) {
              buffer.writeCharCode(_readEscapedChar());
            } else {
              buffer.writeCharCode(nc);
            }
          }

          into.v.add(Leaf(
            v: buffer.toString(),
            line: l,
            column: c,
          ));
      }
    }
  }
}

/// returns a Branch listing every item in the file, even if there's only one item.
Wood parseMultipleWoodslist(String input) {
  return WoodParser(input).parseMultiple();
}

/// if there are multiple items in the file, returns a branch containing them all, otherwise returns the single item.
Wood parseSingleWoodslist(String input) {
  return WoodParser(input).parse();
}

// Stringification
String woodToString(Wood wood) {
  final buffer = StringBuffer();
  _stringifyWood(wood, buffer);
  return buffer.toString();
}

void _stringifyWood(Wood wood, StringBuffer buffer) {
  if (wood is Branch) {
    buffer.writeCharCode(OPEN_PAREN);
    var first = true;
    for (final child in wood.v) {
      if (!first) buffer.writeCharCode(SPACE);
      first = false;
      _stringifyWood(child, buffer);
    }
    buffer.writeCharCode(CLOSE_PAREN);
  } else if (wood is Leaf) {
    final needsQuotes = wood.v.contains(' ') ||
        wood.v.contains('\t') ||
        wood.v.contains('(') ||
        wood.v.contains(')');

    if (needsQuotes) buffer.writeCharCode(QUOTE);
    _writeEscaped(wood.v, buffer);
    if (needsQuotes) buffer.writeCharCode(QUOTE);
  }
}

void _writeEscaped(String str, StringBuffer buffer) {
  for (final char in str.runes) {
    switch (char) {
      case NEWLINE:
        buffer.write('\\n');
        break;
      case CARRIAGE_RETURN:
        buffer.write('\\r');
        break;
      case TAB:
        buffer.write('\\t');
        break;
      case QUOTE:
        buffer.write('\\"');
        break;
      case ESCAPE_SLASH:
        buffer.write('\\\\');
        break;
      default:
        buffer.writeCharCode(char);
    }
  }
}
