```php title="PHP"
<?php
use TreeSitterLanguagePack\LanguagePack;

LanguagePack::init(json_encode(['languages' => ['php']]));

$tree = LanguagePack::parseString('php', '<?php function hello() { echo "world"; } ?>');
echo "Root: " . $tree->rootNodeType() . "\n";
```
