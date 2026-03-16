<?php

declare(strict_types=1);

use PHPUnit\Framework\TestCase;
use TreeSitterLanguagePack\LanguagePack;

final class SmokeTest extends TestCase
{
    private static string $fixturesDir;
    private LanguagePack $pack;

    public static function setUpBeforeClass(): void
    {
        self::$fixturesDir = dirname(__DIR__) . '/fixtures';
    }

    protected function setUp(): void
    {
        $this->pack = new LanguagePack();
    }

    private static function loadFixtures(string $name): array
    {
        $json = file_get_contents(self::$fixturesDir . '/' . $name);
        return json_decode($json, true, 512, JSON_THROW_ON_ERROR);
    }

    /**
     * @dataProvider basicFixtureProvider
     */
    public function testBasicFixture(array $fixture): void
    {
        match ($fixture['test']) {
            'language_count' => $this->assertGreaterThanOrEqual(
                $fixture['expected_min'],
                $this->pack->languageCount(),
                "language_count < expected min {$fixture['expected_min']}"
            ),
            'has_language' => $this->assertSame(
                $fixture['expected'],
                $this->pack->hasLanguage($fixture['language']),
                "has_language({$fixture['language']})"
            ),
            'available_languages' => $this->assertAvailableLanguagesContains(
                $fixture['expected_contains']
            ),
            default => $this->fail("Unknown test type: {$fixture['test']}"),
        };
    }

    public static function basicFixtureProvider(): iterable
    {
        $fixtures = self::loadFixtures('basic.json');
        foreach ($fixtures as $fixture) {
            yield $fixture['name'] => [$fixture];
        }
    }

    private function assertAvailableLanguagesContains(array $expected): void
    {
        $langs = $this->pack->availableLanguages();
        foreach ($expected as $lang) {
            $this->assertContains($lang, $langs, "available_languages missing '{$lang}'");
        }
    }

    public function testParsesPythonCode(): void
    {
        $tree = $this->pack->parseString('python', "def hello(): pass\n");
        $this->assertSame('module', $tree->rootNodeType());
        $this->assertGreaterThanOrEqual(1, $tree->rootChildCount());
        $this->assertFalse($tree->hasErrorNodes());
    }

    public function testErrorsOnInvalidLanguage(): void
    {
        $this->expectException(\RuntimeException::class);
        $this->pack->parseString('nonexistent_xyz_123', 'code');
    }
}
