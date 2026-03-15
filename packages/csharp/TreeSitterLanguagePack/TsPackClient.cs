using System;
using System.Collections.Generic;
using System.Runtime.InteropServices;
using System.Text;
using System.Text.Json;

namespace TreeSitterLanguagePack;

/// <summary>
/// High-level static API for the tree-sitter language pack.
/// Manages a shared registry instance and provides convenient methods for
/// querying languages and processing source code.
/// </summary>
public static class TsPackClient
{
    private static readonly Lazy<IntPtr> SharedRegistry =
        new(() =>
        {
            var reg = NativeMethods.RegistryNew();
            if (reg == IntPtr.Zero)
            {
                InteropUtilities.ThrowIfError();
                throw new TsPackException("failed to create registry");
            }
            return reg;
        }, LazyThreadSafetyMode.ExecutionAndPublication);

    private static IntPtr Registry => SharedRegistry.Value;

    /// <summary>
    /// Get the list of all available language names.
    /// </summary>
    public static string[] AvailableLanguages()
    {
        var count = (int)(nuint)NativeMethods.LanguageCount(Registry);
        var result = new string[count];

        for (var i = 0; i < count; i++)
        {
            var namePtr = NativeMethods.LanguageNameAt(Registry, (UIntPtr)i);
            if (namePtr == IntPtr.Zero)
            {
                continue;
            }

            result[i] = InteropUtilities.Utf8PtrToStringAndFree(namePtr) ?? string.Empty;
        }

        return result;
    }

    /// <summary>
    /// Check whether a language with the given name is available.
    /// </summary>
    public static bool HasLanguage(string name)
    {
        var namePtr = InteropUtilities.StringToUtf8Ptr(name);
        try
        {
            return NativeMethods.HasLanguage(Registry, namePtr);
        }
        finally
        {
            Marshal.FreeHGlobal(namePtr);
        }
    }

    /// <summary>
    /// Get the number of available languages.
    /// </summary>
    public static int LanguageCount()
    {
        return (int)(nuint)NativeMethods.LanguageCount(Registry);
    }

    /// <summary>
    /// Parse source code with the given language and return an opaque tree handle.
    /// The caller must dispose the returned <see cref="ParseTree"/>.
    /// </summary>
    /// <exception cref="TsPackException">Thrown when parsing fails.</exception>
    public static ParseTree Parse(string languageName, string source)
    {
        var namePtr = InteropUtilities.StringToUtf8Ptr(languageName);
        var sourceBytes = Encoding.UTF8.GetBytes(source);
        var sourcePtr = Marshal.AllocHGlobal(sourceBytes.Length);
        Marshal.Copy(sourceBytes, 0, sourcePtr, sourceBytes.Length);

        try
        {
            var treePtr = NativeMethods.ParseString(
                Registry, namePtr, sourcePtr, (UIntPtr)sourceBytes.Length);

            if (treePtr == IntPtr.Zero)
            {
                var errorPtr = NativeMethods.LastError();
                var message = errorPtr != IntPtr.Zero
                    ? Marshal.PtrToStringUTF8(errorPtr) ?? "parse failed"
                    : "parse failed";
                throw new TsPackException(message);
            }

            return new ParseTree(treePtr);
        }
        finally
        {
            Marshal.FreeHGlobal(namePtr);
            Marshal.FreeHGlobal(sourcePtr);
        }
    }

    /// <summary>
    /// Process source code with the given configuration and return analysis results.
    /// </summary>
    /// <exception cref="TsPackException">Thrown when processing fails.</exception>
    public static ProcessResult Process(string source, ProcessConfig config)
    {
        var configJson = JsonSerializer.Serialize(config);
        var configPtr = InteropUtilities.StringToUtf8Ptr(configJson);
        var sourceBytes = Encoding.UTF8.GetBytes(source);
        var sourcePtr = Marshal.AllocHGlobal(sourceBytes.Length);
        Marshal.Copy(sourceBytes, 0, sourcePtr, sourceBytes.Length);

        try
        {
            var resultPtr = NativeMethods.Process(
                Registry, sourcePtr, (UIntPtr)sourceBytes.Length, configPtr);

            if (resultPtr == IntPtr.Zero)
            {
                var errorPtr = NativeMethods.LastError();
                var message = errorPtr != IntPtr.Zero
                    ? Marshal.PtrToStringUTF8(errorPtr) ?? "process failed"
                    : "process failed";
                throw new TsPackException(message);
            }

            var json = InteropUtilities.Utf8PtrToStringAndFree(resultPtr)
                ?? throw new TsPackException("null JSON result from process");

            return JsonSerializer.Deserialize<ProcessResult>(json)
                ?? throw new TsPackException("failed to deserialize process result");
        }
        finally
        {
            Marshal.FreeHGlobal(configPtr);
            Marshal.FreeHGlobal(sourcePtr);
        }
    }
}

/// <summary>
/// An opaque handle to a parsed syntax tree. Must be disposed to free native memory.
/// </summary>
public sealed class ParseTree : IDisposable
{
    private IntPtr _handle;
    private bool _disposed;

    internal ParseTree(IntPtr handle)
    {
        _handle = handle;
    }

    /// <summary>
    /// Get the type name of the root node.
    /// </summary>
    public string? RootNodeType()
    {
        ObjectDisposedException.ThrowIf(_disposed, this);
        return InteropUtilities.Utf8PtrToStringAndFree(NativeMethods.TreeRootNodeType(_handle));
    }

    /// <summary>
    /// Get the number of named children of the root node.
    /// </summary>
    public uint RootChildCount()
    {
        ObjectDisposedException.ThrowIf(_disposed, this);
        return NativeMethods.TreeRootChildCount(_handle);
    }

    /// <summary>
    /// Check whether the tree contains a node with the given type name.
    /// </summary>
    public bool ContainsNodeType(string nodeType)
    {
        ObjectDisposedException.ThrowIf(_disposed, this);
        var ptr = InteropUtilities.StringToUtf8Ptr(nodeType);
        try
        {
            return NativeMethods.TreeContainsNodeType(_handle, ptr);
        }
        finally
        {
            Marshal.FreeHGlobal(ptr);
        }
    }

    /// <summary>
    /// Check whether the tree contains any ERROR or MISSING nodes.
    /// </summary>
    public bool HasErrorNodes()
    {
        ObjectDisposedException.ThrowIf(_disposed, this);
        return NativeMethods.TreeHasErrorNodes(_handle);
    }

    /// <summary>
    /// Return the S-expression representation of the tree.
    /// </summary>
    public string? ToSexp()
    {
        ObjectDisposedException.ThrowIf(_disposed, this);
        return InteropUtilities.Utf8PtrToStringAndFree(NativeMethods.TreeToSexp(_handle));
    }

    /// <summary>
    /// Return the count of ERROR and MISSING nodes in the tree.
    /// </summary>
    public int ErrorCount()
    {
        ObjectDisposedException.ThrowIf(_disposed, this);
        return (int)(nuint)NativeMethods.TreeErrorCount(_handle);
    }

    /// <inheritdoc/>
    public void Dispose()
    {
        if (!_disposed)
        {
            if (_handle != IntPtr.Zero)
            {
                NativeMethods.TreeFree(_handle);
                _handle = IntPtr.Zero;
            }
            _disposed = true;
        }
    }
}
