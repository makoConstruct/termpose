## woodslist dart

A dart library for parsing the s-expression type serialization format, [woodslist](../woodslist.md).

```dart
import 'pkg:woodslist/woodslist.dart';

void main() {
    final Wood wood = parseWoodslist('(test "hello world" (nested 1 2 3))');
    
    assert(wood == Wood.from(["test", "hello world", ["nested", "1", "2", "3"]]));
    
    print("${woodToString(wood)} parsed successfully");
}
```