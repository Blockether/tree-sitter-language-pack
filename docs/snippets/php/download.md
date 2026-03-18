```php title="PHP"
<?php
use TreeSitterLanguagePack\LanguagePack;

LanguagePack::init(json_encode(['languages' => ['php', 'javascript']]));
LanguagePack::download(['python', 'rust']);

print_r(LanguagePack::downloadedLanguages());
```
