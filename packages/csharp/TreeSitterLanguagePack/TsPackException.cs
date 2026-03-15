using System;

namespace TreeSitterLanguagePack;

/// <summary>
/// Exception thrown when a native FFI call to the tree-sitter language pack library fails.
/// </summary>
public sealed class TsPackException : Exception
{
    /// <summary>
    /// Initializes a new instance of <see cref="TsPackException"/> with the specified error message.
    /// </summary>
    public TsPackException(string message) : base(message)
    {
    }

    /// <summary>
    /// Initializes a new instance of <see cref="TsPackException"/> with a message and inner exception.
    /// </summary>
    public TsPackException(string message, Exception innerException) : base(message, innerException)
    {
    }
}
