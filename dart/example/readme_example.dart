import '../woodslist.dart';

void main() {
  final Wood wood = parseWoodslist('(test "hello world" (nested 1 2 3))');

  assert(wood ==
      Wood.from([
        "test",
        "hello world",
        ["nested", "1", "2", "3"]
      ]));

  print("${woodToString(wood)} parsed successfully");
}
