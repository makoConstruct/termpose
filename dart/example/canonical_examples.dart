import '../woodslist.dart';

void main() {
  assert(parseWoodslist('("quoted"then)') == Wood.from(["quoted", "then"]));

  assert(parseWoodslist('("first""second")') == Wood.from(["first", "second"]));

  assert(parseWoodslist('"first""second"') == Wood.from(["first", "second"]));

  assert(parseWoodslist('''
(a "
  first
  second
")
''') == Wood.from(["a", "  first\n  second\n"]));

  print("all canonical tests passed");
}
