using System;
using System.Runtime.InteropServices;
using System.Text;

namespace TreeSitterLanguagePack;

/// <summary>
/// UTF-8 string marshaling helpers for P/Invoke interop with the Rust FFI layer.
/// </summary>
internal static class InteropUtilities
{
    /// <summary>
    /// Allocate a native UTF-8 null-terminated string from a managed string.
    /// The caller must free the returned pointer with <see cref="Marshal.FreeHGlobal"/>.
    /// </summary>
    internal static IntPtr StringToUtf8Ptr(string value)
    {
        var bytes = Encoding.UTF8.GetBytes(value);
        var ptr = Marshal.AllocHGlobal(bytes.Length + 1);
        Marshal.Copy(bytes, 0, ptr, bytes.Length);
        Marshal.WriteByte(ptr, bytes.Length, 0); // null terminator
        return ptr;
    }

    /// <summary>
    /// Read a null-terminated UTF-8 string from a native pointer.
    /// Returns null if the pointer is <see cref="IntPtr.Zero"/>.
    /// </summary>
    internal static string? Utf8PtrToString(IntPtr ptr)
    {
        if (ptr == IntPtr.Zero)
        {
            return null;
        }

        return Marshal.PtrToStringUTF8(ptr);
    }

    /// <summary>
    /// Read a null-terminated UTF-8 string from a native pointer, then free the
    /// pointer using <see cref="NativeMethods.FreeString"/>.
    /// Returns null if the pointer is <see cref="IntPtr.Zero"/>.
    /// </summary>
    internal static string? Utf8PtrToStringAndFree(IntPtr ptr)
    {
        if (ptr == IntPtr.Zero)
        {
            return null;
        }

        try
        {
            return Marshal.PtrToStringUTF8(ptr);
        }
        finally
        {
            NativeMethods.FreeString(ptr);
        }
    }

    /// <summary>
    /// Check for an error from the native library and throw an exception if one exists.
    /// </summary>
    internal static void ThrowIfError()
    {
        var errorPtr = NativeMethods.LastError();
        if (errorPtr != IntPtr.Zero)
        {
            var message = Marshal.PtrToStringUTF8(errorPtr) ?? "unknown native error";
            throw new TsPackException(message);
        }
    }
}
