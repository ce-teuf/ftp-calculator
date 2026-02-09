using System;
using System.Runtime.InteropServices;

namespace FtpAddIn
{
    /// <summary>
    /// P/Invoke declarations for ftp_core_bindings_c.dll (Rust cdylib).
    /// </summary>
    internal static class FtpNative
    {
        private const string DllName = "ftp_core_bindings_c";

        [DllImport(DllName, CallingConvention = CallingConvention.Cdecl)]
        public static extern IntPtr ftp_create(
            double[] outstanding, int outs_rows,
            double[] profiles, int prof_rows, int prof_cols,
            double[] rates, int rate_rows, int rate_cols);

        [DllImport(DllName, CallingConvention = CallingConvention.Cdecl)]
        public static extern void ftp_free(IntPtr handle);

        [DllImport(DllName, CallingConvention = CallingConvention.Cdecl)]
        public static extern int ftp_compute(IntPtr handle, int method);

        [DllImport(DllName, CallingConvention = CallingConvention.Cdecl)]
        public static extern int ftp_get_dims(IntPtr handle, out int rows, out int cols);

        [DllImport(DllName, CallingConvention = CallingConvention.Cdecl)]
        public static extern int ftp_get_stock_amort(IntPtr handle, double[] buf, int buf_len);

        [DllImport(DllName, CallingConvention = CallingConvention.Cdecl)]
        public static extern int ftp_get_stock_instal(IntPtr handle, double[] buf, int buf_len);

        [DllImport(DllName, CallingConvention = CallingConvention.Cdecl)]
        public static extern int ftp_get_varstock_amort(IntPtr handle, double[] buf, int buf_len);

        [DllImport(DllName, CallingConvention = CallingConvention.Cdecl)]
        public static extern int ftp_get_varstock_instal(IntPtr handle, double[] buf, int buf_len);

        [DllImport(DllName, CallingConvention = CallingConvention.Cdecl)]
        public static extern int ftp_get_ftp_rate(IntPtr handle, double[] buf, int buf_len);

        [DllImport(DllName, CallingConvention = CallingConvention.Cdecl)]
        public static extern int ftp_get_ftp_int(IntPtr handle, double[] buf, int buf_len);

        [DllImport(DllName, CallingConvention = CallingConvention.Cdecl)]
        public static extern int ftp_get_market_rate(IntPtr handle, double[] buf, int buf_len);

        [DllImport(DllName, CallingConvention = CallingConvention.Cdecl)]
        public static extern int ftp_get_last_error(byte[] buf, int buf_len);

        /// <summary>
        /// Returns the last error message from the Rust library.
        /// </summary>
        public static string GetLastError()
        {
            var buf = new byte[512];
            ftp_get_last_error(buf, buf.Length);
            int end = Array.IndexOf(buf, (byte)0);
            if (end < 0) end = buf.Length;
            return System.Text.Encoding.UTF8.GetString(buf, 0, end);
        }
    }
}
