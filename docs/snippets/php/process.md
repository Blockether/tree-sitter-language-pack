```php title="PHP"
<?php
use TreeSitterLanguagePack\LanguagePack;

$result = LanguagePack::process(
    '<?php namespace App; class Controller { public function index() {} }',
    json_encode(['language' => 'php', 'structure' => true, 'imports' => true])
);
echo $result;
```
