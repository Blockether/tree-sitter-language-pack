package io.github.treesitter.languagepack;

/**
 * Thrown when a requested language is not available in the registry.
 *
 * <p>This is a subclass of {@link IllegalArgumentException}, so existing
 * {@code catch (IllegalArgumentException)} blocks will still work.</p>
 */
public class LanguageNotFoundException extends IllegalArgumentException {

    private final String languageName;

    /**
     * Creates a new exception for the given language name.
     *
     * @param languageName the name of the language that was not found
     */
    public LanguageNotFoundException(String languageName) {
        super("Language not found: " + languageName);
        this.languageName = languageName;
    }

    /**
     * Creates a new exception for the given language name with a detail message.
     *
     * @param languageName the name of the language that was not found
     * @param detail       additional detail (e.g. FFI error message)
     */
    public LanguageNotFoundException(String languageName, String detail) {
        super("Language not found: " + languageName + " (" + detail + ")");
        this.languageName = languageName;
    }

    /**
     * Returns the name of the language that was not found.
     *
     * @return the language name
     */
    public String getLanguageName() {
        return languageName;
    }
}
