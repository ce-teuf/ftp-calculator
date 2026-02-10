using System;
using System.IO;
using System.Reflection;
using System.Runtime.InteropServices;
using ExcelDna.Integration;
using ExcelDna.Integration.Rtd;

namespace FtpAddIn
{
    /// <summary>
    /// Loads native DLLs embedded in the XLL at runtime.
    /// </summary>
    public static class NativeLibraryLoader
    {
        private const string DllName = "ftp_calculator_bindings_c.dll";

        [DllImport("kernel32.dll", SetLastError = true)]
        private static extern IntPtr LoadLibrary(string lpFileName);

        /// <summary>
        /// Extracts and loads the native Rust DLL from embedded resources.
        /// Called automatically by ExcelDna when the add-in loads.
        /// </summary>
        public static void LoadNativeDll()
        {
            try
            {
                // Get a temp directory for this add-in
                var tempDir = Path.Combine(Path.GetTempPath(), "FtpAddIn");
                Directory.CreateDirectory(tempDir);

                var dllPath = Path.Combine(tempDir, DllName);

                // Check if DLL already exists next to the XLL (non-packed scenario)
                var xllPath = (string)XlCall.Excel(XlCall.xlGetName);
                var xllDir = Path.GetDirectoryName(xllPath);
                var dllNextToXll = Path.Combine(xllDir, DllName);

                if (File.Exists(dllNextToXll))
                {
                    // DLL is next to XLL, use it directly
                    var handle = LoadLibrary(dllNextToXll);
                    if (handle == IntPtr.Zero)
                    {
                        var error = Marshal.GetLastWin32Error();
                        throw new Exception($"Failed to load {DllName} from {dllNextToXll}. Error code: {error}");
                    }
                }
                else
                {
                    // Try to extract from embedded resources
                    var assembly = Assembly.GetExecutingAssembly();
                    var resourceName = Array.Find(assembly.GetManifestResourceNames(), n => n.EndsWith(DllName));

                    if (resourceName != null)
                    {
                        using (var stream = assembly.GetManifestResourceStream(resourceName))
                        {
                            if (stream != null)
                            {
                                var resourceBytes = new byte[stream.Length];
                                stream.Read(resourceBytes, 0, (int)stream.Length);

                                if (!File.Exists(dllPath) || !FilesAreEqual(dllPath, resourceBytes))
                                {
                                    File.WriteAllBytes(dllPath, resourceBytes);
                                }

                                var handle = LoadLibrary(dllPath);
                                if (handle == IntPtr.Zero)
                                {
                                    var error = Marshal.GetLastWin32Error();
                                    throw new Exception($"Failed to load {DllName}. Error code: {error}");
                                }
                            }
                        }
                    }
                    else
                    {
                        throw new Exception($"Native DLL not found. Expected at: {dllNextToXll} or as embedded resource.");
                    }
                }
            }
            catch (Exception ex)
            {
                // Log error but don't crash the add-in
                System.Diagnostics.Debug.WriteLine($"Error loading native DLL: {ex.Message}");
                throw; // Re-throw so Excel shows the error
            }
        }

        private static bool FilesAreEqual(string filePath, byte[] newBytes)
        {
            try
            {
                var existingBytes = File.ReadAllBytes(filePath);
                if (existingBytes.Length != newBytes.Length)
                    return false;

                for (int i = 0; i < existingBytes.Length; i++)
                {
                    if (existingBytes[i] != newBytes[i])
                        return false;
                }
                return true;
            }
            catch
            {
                return false;
            }
        }
    }

    /// <summary>
    /// Add-in initialization class.
    /// </summary>
    public class AddInInitializer : IExcelAddIn
    {
        public void AutoOpen()
        {
            // Load native DLL when add-in opens
            NativeLibraryLoader.LoadNativeDll();
        }

        public void AutoClose()
        {
            // Cleanup if needed
        }
    }
}
