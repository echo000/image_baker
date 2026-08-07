// TextureSmith C# example
// Requires: texture_smith_core.dll (built with `cargo build -p texture_smith_core --features library --release`)
// Target framework: .NET 6+

using System;
using System.Collections.Generic;
using System.IO;
using System.Runtime.InteropServices;

// ── Wrapper ──────────────────────────────────────────────────────────────────

public enum TsImageFormat
{
    Png  = 0,
    Tga  = 1,
    Tiff = 2,
    Dds  = 3,
}

public enum TsDdsFormat
{
    Bc1Unorm       = 0,
    Bc1UnormSrgb   = 1,
    Bc3Unorm       = 2,
    Bc3UnormSrgb   = 3,
    Bc4Unorm       = 4,
    Bc5Unorm       = 5,
    Bc7Unorm       = 6,
    Bc7UnormSrgb   = 7,
    Rgba8Unorm     = 8,
    Rgba8UnormSrgb = 9,
    Bgra8Unorm     = 10,
    Bgra8UnormSrgb = 11,
    R8Unorm        = 12,
    R8G8Unorm      = 13,
    Rgba16Float    = 14,
    R32Float       = 15,
}

/// <summary>
/// One output image from a shader pass. Pixel data is RGBA8, row-major.
/// </summary>
public sealed class TsImage
{
    public uint   Width  { get; }
    public uint   Height { get; }
    public byte[] Pixels { get; }   // RGBA8, Width * Height * 4 bytes

    internal TsImage(uint w, uint h, byte[] pixels)
    {
        Width  = w;
        Height = h;
        Pixels = pixels;
    }
}

/// <summary>
/// Wraps the native TsResult handle. Dispose when done.
/// </summary>
public sealed class TsResult : IDisposable
{
    private IntPtr _handle;

    internal TsResult(IntPtr handle) => _handle = handle;

    public uint Count => Native.ts_result_count(_handle);

    /// <summary>Copies output <paramref name="index"/> pixel data into a managed TsImage.</summary>
    public TsImage GetImage(uint index)
    {
        uint   w      = Native.ts_result_width(_handle, index);
        uint   h      = Native.ts_result_height(_handle, index);
        nuint  len    = Native.ts_result_len(_handle, index);
        IntPtr srcPtr = Native.ts_result_data(_handle, index);

        byte[] pixels = new byte[(int)len];
        Marshal.Copy(srcPtr, pixels, 0, pixels.Length);
        return new TsImage(w, h, pixels);
    }

    /// <summary>
    /// Save all outputs to <paramref name="outputDir"/>.
    /// Throws on failure.
    /// </summary>
    public void Save(
        string        outputDir,
        TsImageFormat format    = TsImageFormat.Png,
        TsDdsFormat   ddsFormat = TsDdsFormat.Rgba8Unorm)
    {
        IntPtr err = Native.ts_result_save(_handle, outputDir, (uint)format, (uint)ddsFormat);
        if (err != IntPtr.Zero)
        {
            string msg = Marshal.PtrToStringAnsi(err) ?? "unknown error";
            Native.ts_free_string(err);
            throw new InvalidOperationException($"ts_result_save failed: {msg}");
        }
    }

    /// <summary>
    /// Save a single output at <paramref name="index"/> to <paramref name="outputDir"/>.
    /// Throws on failure.
    /// </summary>
    public void Save(
        uint          index,
        string        outputDir,
        TsImageFormat format    = TsImageFormat.Png,
        TsDdsFormat   ddsFormat = TsDdsFormat.Rgba8Unorm)
    {
        IntPtr err = Native.ts_result_save_one(_handle, index, outputDir, (uint)format, (uint)ddsFormat);
        if (err != IntPtr.Zero)
        {
            string msg = Marshal.PtrToStringAnsi(err) ?? "unknown error";
            Native.ts_free_string(err);
            throw new InvalidOperationException($"ts_result_save_one failed: {msg}");
        }
    }

    public void Dispose()
    {
        if (_handle != IntPtr.Zero)
        {
            Native.ts_result_free(_handle);
            _handle = IntPtr.Zero;
        }
    }
}

/// <summary>
/// Main entry point into the Texture Smith core library.
/// </summary>
public static class TextureSmith
{
    /// <summary>
    /// Process a set of images through the named shader.
    /// </summary>
    /// <param name="shadersDir">
    ///   Path to the directory that contains shader subdirectories
    ///   (each with a shader.wgsl and config.toml).
    /// </param>
    /// <param name="shaderName">
    ///   The <c>[shader].name</c> value from the target shader's config.toml.
    /// </param>
    /// <param name="inputs">
    ///   List of (raw file bytes, extension) pairs — e.g. (File.ReadAllBytes("a.png"), "png").
    ///   The extension tells the library how to decode the bytes.
    /// </param>
    /// <param name="parameters">Optional shader parameter overrides.</param>
    /// <returns>A <see cref="TsResult"/> that must be disposed when done.</returns>
    public static TsResult Process(
        string                                    shadersDir,
        string                                    shaderName,
        IReadOnlyList<(byte[] Data, string Ext)>  inputs,
        IReadOnlyDictionary<string, float>?       parameters = null)
    {
        int imageCount = inputs.Count;

        // Pin each input byte array and collect the pointers.
        var    handles   = new GCHandle[imageCount];
        var    dataPtrs  = new IntPtr[imageCount];
        var    dataLens  = new nuint[imageCount];
        IntPtr[] extPtrs = new IntPtr[imageCount];

        try
        {
            for (int i = 0; i < imageCount; i++)
            {
                handles[i]  = GCHandle.Alloc(inputs[i].Data, GCHandleType.Pinned);
                dataPtrs[i] = handles[i].AddrOfPinnedObject();
                dataLens[i] = (nuint)inputs[i].Data.Length;
                extPtrs[i]  = Marshal.StringToHGlobalAnsi(inputs[i].Ext);
            }

            // Marshal parameter names and values.
            parameters ??= new Dictionary<string, float>();
            int       paramCount  = parameters.Count;
            IntPtr[]  paramNames  = new IntPtr[paramCount];
            float[]   paramValues = new float[paramCount];

            int pi = 0;
            foreach (var (name, value) in parameters)
            {
                paramNames[pi]  = Marshal.StringToHGlobalAnsi(name);
                paramValues[pi] = value;
                pi++;
            }

            try
            {
                IntPtr result = Native.ts_process(
                    shadersDir,
                    shaderName,
                    dataPtrs,
                    dataLens,
                    extPtrs,
                    (uint)imageCount,
                    paramNames,
                    paramValues,
                    (uint)paramCount);

                if (result == IntPtr.Zero)
                {
                    IntPtr errPtr = Native.ts_last_error();
                    string err    = Marshal.PtrToStringAnsi(errPtr) ?? "unknown error";
                    throw new InvalidOperationException($"ts_process failed: {err}");
                }

                return new TsResult(result);
            }
            finally
            {
                foreach (IntPtr p in paramNames)
                    if (p != IntPtr.Zero) Marshal.FreeHGlobal(p);
            }
        }
        finally
        {
            for (int i = 0; i < imageCount; i++)
            {
                if (handles[i].IsAllocated) handles[i].Free();
                if (extPtrs[i] != IntPtr.Zero) Marshal.FreeHGlobal(extPtrs[i]);
            }
        }
    }
}

// ── Native bindings (internal) ───────────────────────────────────────────────

internal static class Native
{
    const string Lib = "texture_smith_core";

    [DllImport(Lib, CharSet = CharSet.Ansi)]
    public static extern IntPtr ts_process(
        string   shadersDir,
        string   shaderName,
        IntPtr[] inputData,
        nuint[]  inputLens,
        IntPtr[] extensions,
        uint     imageCount,
        IntPtr[] paramNames,
        float[]  paramValues,
        uint     paramCount);

    [DllImport(Lib)] public static extern uint   ts_result_count(IntPtr result);
    [DllImport(Lib)] public static extern IntPtr ts_result_data(IntPtr result, uint index);
    [DllImport(Lib)] public static extern nuint  ts_result_len(IntPtr result, uint index);
    [DllImport(Lib)] public static extern uint   ts_result_width(IntPtr result, uint index);
    [DllImport(Lib)] public static extern uint   ts_result_height(IntPtr result, uint index);

    [DllImport(Lib, CharSet = CharSet.Ansi)]
    public static extern IntPtr ts_result_save(IntPtr result, string outputDir, uint format, uint ddsFormat);

    [DllImport(Lib, CharSet = CharSet.Ansi)]
    public static extern IntPtr ts_result_save_one(IntPtr result, uint index, string outputDir, uint format, uint ddsFormat);

    [DllImport(Lib)] public static extern void   ts_result_free(IntPtr result);
    [DllImport(Lib)] public static extern void   ts_free_string(IntPtr ptr);
    [DllImport(Lib)] public static extern IntPtr ts_last_error();
}

// ── Example usage ────────────────────────────────────────────────────────────

class Program
{
    static void Main(string[] args)
    {
        // Paths — adjust these for your setup.
        string shadersDir = @"C:\path\to\shaders";     // folder with shader subdirs
        string outputDir  = @"C:\path\to\output";
        string shaderName = "My Shader";               // must match [shader].name in config.toml

        // --- Example 1: process and save to disk ---
        Console.WriteLine("Processing...");

        var inputs = new List<(byte[], string)>
        {
            (File.ReadAllBytes(@"C:\path\to\diffuse.png"),  "png"),
            (File.ReadAllBytes(@"C:\path\to\normal.dds"),   "dds"),
        };

        var parameters = new Dictionary<string, float>
        {
            ["roughness"] = 0.8f,
            ["metallic"]  = 0.0f,
        };

        using TsResult result = TextureSmith.Process(shadersDir, shaderName, inputs, parameters);

        Console.WriteLine($"Got {result.Count} output(s).");

        // Save all outputs as PNG files in outputDir.
        result.Save(outputDir, TsImageFormat.Png);
        Console.WriteLine($"Saved to {outputDir}");

        // --- Example 2: grab raw RGBA8 pixel data instead ---
        for (uint i = 0; i < result.Count; i++)
        {
            TsImage img = result.GetImage(i);
            Console.WriteLine($"Output {i}: {img.Width}x{img.Height}, {img.Pixels.Length} bytes RGBA8");

            // img.Pixels is a plain byte[] you can hand to any image library,
            // e.g. ImageSharp, SkiaSharp, System.Drawing, etc.
        }
    }
}
