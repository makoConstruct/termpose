import 'package:woodslist/woodslist.dart';
import 'package:test/test.dart';

void main() {
  test('readme example', () {
    final Wood wood =
        parseSingleWoodslist('(test "hello world" (nested 1 2 3))');

    expect(
      wood,
      equals(
        Wood.from([
          "test",
          "hello world",
          ["nested", "1", "2", "3"]
        ]),
      ),
    );

    print("${woodToString(wood)} parsed successfully");
  });
}
