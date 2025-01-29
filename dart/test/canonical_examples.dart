import 'package:woodslist/woodslist.dart';
import 'package:test/test.dart';

void main() {
  test('idk I\'m just going to test possible edge cases', () {
    expect(
      parseMultipleWoodslist('("quoted"then)'),
      equals(Wood.from([
        ["quoted", "then"]
      ])),
    );

    expect(
      parseMultipleWoodslist('("first""second")'),
      equals(Wood.from([
        ["first", "second"]
      ])),
    );

    expect(
      parseSingleWoodslist('"first""second"'),
      equals(Wood.from(["first", "second"])),
    );

    expect(
      parseMultipleWoodslist('''
(a "
  first
  second
")
'''),
      equals(Wood.from([
        ["a", "  first\n  second\n"]
      ])),
    );
  });
}
