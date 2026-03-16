import { describe, expect, it, beforeAll } from "vitest";
import init, {
	availableLanguages,
	hasLanguage,
	languageCount,
	parseString,
	treeRootNodeType,
	treeHasErrorNodes,
} from "@kreuzberg/tree-sitter-language-pack-wasm";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";

interface BasicFixture {
	name: string;
	test: string;
	language?: string;
	expected?: boolean;
	expected_min?: number;
	expected_contains?: string[];
}

const fixturesDir = resolve(import.meta.dirname, "..", "fixtures");

function loadFixtures<T>(name: string): T[] {
	return JSON.parse(readFileSync(resolve(fixturesDir, name), "utf-8"));
}

describe("wasm smoke tests", () => {
	beforeAll(async () => {
		await init();
	});
	describe("basic fixtures", () => {
		const fixtures = loadFixtures<BasicFixture>("basic.json");

		for (const fixture of fixtures) {
			it(fixture.name, () => {
				switch (fixture.test) {
					case "language_count": {
						const count = languageCount();
						expect(count).toBeGreaterThanOrEqual(fixture.expected_min!);
						break;
					}
					case "has_language": {
						const result = hasLanguage(fixture.language!);
						expect(result).toBe(fixture.expected);
						break;
					}
					case "available_languages": {
						const langs = availableLanguages();
						for (const lang of fixture.expected_contains!) {
							expect(langs).toContain(lang);
						}
						break;
					}
					default:
						throw new Error(`Unknown test type: ${fixture.test}`);
				}
			});
		}
	});

	describe("parse validation", () => {
		it("parses Python code", () => {
			const tree = parseString("python", "def hello(): pass\n");
			expect(treeRootNodeType(tree)).toBe("module");
			expect(treeHasErrorNodes(tree)).toBe(false);
		});

		it("throws on invalid language", () => {
			expect(() => parseString("nonexistent_xyz_123", "code")).toThrow();
		});
	});
});
