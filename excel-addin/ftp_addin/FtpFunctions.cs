using System;
using ExcelDna.Integration;

namespace FtpAddIn
{
    /// <summary>
    /// Excel UDF functions exposed via Excel-DNA.
    /// Each function reads Excel ranges, calls Rust via P/Invoke, and returns results.
    /// </summary>
    public static class FtpFunctions
    {
        // --------------------------------------------------------------------
        // Full compute — returns all 7 output matrices stacked vertically
        // --------------------------------------------------------------------

        [ExcelFunction(
            Name = "FTP_COMPUTE_STOCK",
            Description = "Compute FTP using the Stock method. Returns all output matrices.",
            Category = "FTP Calculator")]
        public static object FtpComputeStock(
            [ExcelArgument(Name = "outstanding", Description = "Column vector of outstanding amounts")] object outstanding,
            [ExcelArgument(Name = "profiles", Description = "Profiles matrix")] object profiles,
            [ExcelArgument(Name = "rates", Description = "Rates matrix")] object rates)
        {
            if (!TryGetRange(outstanding, out object[,] o)) return ExcelError.ExcelErrorValue;
            if (!TryGetRange(profiles, out object[,] p)) return ExcelError.ExcelErrorValue;
            if (!TryGetRange(rates, out object[,] r)) return ExcelError.ExcelErrorValue;
            return ComputeAll(o, p, r, method: 0);
        }

        [ExcelFunction(
            Name = "FTP_COMPUTE_FLUX",
            Description = "Compute FTP using the Flux method. Returns all output matrices.",
            Category = "FTP Calculator")]
        public static object FtpComputeFlux(
            [ExcelArgument(Name = "outstanding", Description = "Column vector of outstanding amounts")] object outstanding,
            [ExcelArgument(Name = "profiles", Description = "Profiles matrix")] object profiles,
            [ExcelArgument(Name = "rates", Description = "Rates matrix")] object rates)
        {
            if (!TryGetRange(outstanding, out object[,] o)) return ExcelError.ExcelErrorValue;
            if (!TryGetRange(profiles, out object[,] p)) return ExcelError.ExcelErrorValue;
            if (!TryGetRange(rates, out object[,] r)) return ExcelError.ExcelErrorValue;
            return ComputeAll(o, p, r, method: 1);
        }

        // --------------------------------------------------------------------
        // Individual getters
        // --------------------------------------------------------------------

        [ExcelFunction(Name = "FTP_STOCK_AMORT", Description = "Stock amortization matrix", Category = "FTP Calculator")]
        public static object FtpStockAmort(
            [ExcelArgument(Name = "outstanding", Description = "Column vector of outstanding amounts")] object outstanding,
            [ExcelArgument(Name = "profiles", Description = "Profiles matrix")] object profiles,
            [ExcelArgument(Name = "rates", Description = "Rates matrix")] object rates,
            [ExcelArgument(Name = "method", Description = "0=Stock, 1=Flux")] object method)
        {
            if (!TryValidateArgs(outstanding, profiles, rates, method, out var args)) return ExcelError.ExcelErrorValue;
            return ComputeSingle(args.o, args.p, args.r, args.m, FtpNative.ftp_get_stock_amort);
        }

        [ExcelFunction(Name = "FTP_STOCK_INSTAL", Description = "Stock installment matrix", Category = "FTP Calculator")]
        public static object FtpStockInstal(
            [ExcelArgument(Name = "outstanding", Description = "Column vector of outstanding amounts")] object outstanding,
            [ExcelArgument(Name = "profiles", Description = "Profiles matrix")] object profiles,
            [ExcelArgument(Name = "rates", Description = "Rates matrix")] object rates,
            [ExcelArgument(Name = "method", Description = "0=Stock, 1=Flux")] object method)
        {
            if (!TryValidateArgs(outstanding, profiles, rates, method, out var args)) return ExcelError.ExcelErrorValue;
            return ComputeSingle(args.o, args.p, args.r, args.m, FtpNative.ftp_get_stock_instal);
        }

        [ExcelFunction(Name = "FTP_VARSTOCK_AMORT", Description = "Variation stock amortization matrix", Category = "FTP Calculator")]
        public static object FtpVarstockAmort(
            [ExcelArgument(Name = "outstanding", Description = "Column vector of outstanding amounts")] object outstanding,
            [ExcelArgument(Name = "profiles", Description = "Profiles matrix")] object profiles,
            [ExcelArgument(Name = "rates", Description = "Rates matrix")] object rates,
            [ExcelArgument(Name = "method", Description = "0=Stock, 1=Flux")] object method)
        {
            if (!TryValidateArgs(outstanding, profiles, rates, method, out var args)) return ExcelError.ExcelErrorValue;
            return ComputeSingle(args.o, args.p, args.r, args.m, FtpNative.ftp_get_varstock_amort);
        }

        [ExcelFunction(Name = "FTP_VARSTOCK_INSTAL", Description = "Variation stock installment matrix", Category = "FTP Calculator")]
        public static object FtpVarstockInstal(
            [ExcelArgument(Name = "outstanding", Description = "Column vector of outstanding amounts")] object outstanding,
            [ExcelArgument(Name = "profiles", Description = "Profiles matrix")] object profiles,
            [ExcelArgument(Name = "rates", Description = "Rates matrix")] object rates,
            [ExcelArgument(Name = "method", Description = "0=Stock, 1=Flux")] object method)
        {
            if (!TryValidateArgs(outstanding, profiles, rates, method, out var args)) return ExcelError.ExcelErrorValue;
            return ComputeSingle(args.o, args.p, args.r, args.m, FtpNative.ftp_get_varstock_instal);
        }

        [ExcelFunction(Name = "FTP_FTP_RATE", Description = "FTP rate matrix", Category = "FTP Calculator")]
        public static object FtpFtpRate(
            [ExcelArgument(Name = "outstanding", Description = "Column vector of outstanding amounts")] object outstanding,
            [ExcelArgument(Name = "profiles", Description = "Profiles matrix")] object profiles,
            [ExcelArgument(Name = "rates", Description = "Rates matrix")] object rates,
            [ExcelArgument(Name = "method", Description = "0=Stock, 1=Flux")] object method)
        {
            if (!TryValidateArgs(outstanding, profiles, rates, method, out var args)) return ExcelError.ExcelErrorValue;
            return ComputeSingle(args.o, args.p, args.r, args.m, FtpNative.ftp_get_ftp_rate);
        }

        [ExcelFunction(Name = "FTP_FTP_INT", Description = "FTP interest matrix", Category = "FTP Calculator")]
        public static object FtpFtpInt(
            [ExcelArgument(Name = "outstanding", Description = "Column vector of outstanding amounts")] object outstanding,
            [ExcelArgument(Name = "profiles", Description = "Profiles matrix")] object profiles,
            [ExcelArgument(Name = "rates", Description = "Rates matrix")] object rates,
            [ExcelArgument(Name = "method", Description = "0=Stock, 1=Flux")] object method)
        {
            if (!TryValidateArgs(outstanding, profiles, rates, method, out var args)) return ExcelError.ExcelErrorValue;
            return ComputeSingle(args.o, args.p, args.r, args.m, FtpNative.ftp_get_ftp_int);
        }

        [ExcelFunction(Name = "FTP_MARKET_RATE", Description = "Market rate matrix", Category = "FTP Calculator")]
        public static object FtpMarketRate(
            [ExcelArgument(Name = "outstanding", Description = "Column vector of outstanding amounts")] object outstanding,
            [ExcelArgument(Name = "profiles", Description = "Profiles matrix")] object profiles,
            [ExcelArgument(Name = "rates", Description = "Rates matrix")] object rates,
            [ExcelArgument(Name = "method", Description = "0=Stock, 1=Flux")] object method)
        {
            if (!TryValidateArgs(outstanding, profiles, rates, method, out var args)) return ExcelError.ExcelErrorValue;
            return ComputeSingle(args.o, args.p, args.r, args.m, FtpNative.ftp_get_market_rate);
        }

        // --------------------------------------------------------------------
        // Helpers
        // --------------------------------------------------------------------

        private delegate int GetterDelegate(IntPtr handle, double[] buf, int buf_len);

        private static bool IsMissing(object arg)
        {
            return arg is ExcelMissing || arg is ExcelEmpty;
        }

        private static bool TryGetRange(object arg, out object[,] range)
        {
            range = null;
            if (IsMissing(arg)) return false;
            if (arg is object[,] arr)
            {
                range = arr;
                return true;
            }
            // Single cell value — wrap into 1x1 array
            range = new object[1, 1] { { arg } };
            return true;
        }

        private static bool TryParseMethod(object method, out int result)
        {
            result = 0;
            if (IsMissing(method)) return false;
            try
            {
                result = Convert.ToInt32(method);
                return true;
            }
            catch
            {
                return false;
            }
        }

        private static bool TryValidateArgs(object outstanding, object profiles, object rates, object method,
            out (object[,] o, object[,] p, object[,] r, int m) args)
        {
            args = default;
            if (!TryGetRange(outstanding, out var o)) return false;
            if (!TryGetRange(profiles, out var p)) return false;
            if (!TryGetRange(rates, out var r)) return false;
            if (!TryParseMethod(method, out int m)) return false;
            args = (o, p, r, m);
            return true;
        }

        /// <summary>
        /// Runs compute and returns a single output matrix.
        /// </summary>
        private static object ComputeSingle(
            object[,] outstanding, object[,] profiles, object[,] rates,
            int method, GetterDelegate getter)
        {
            try
            {
                var (outs, oRows) = FlattenColumn(outstanding);
                var (prof, pRows, pCols) = FlattenMatrix(profiles);
                var (rate, rRows, rCols) = FlattenMatrix(rates);

                IntPtr h = FtpNative.ftp_create(outs, oRows, prof, pRows, pCols, rate, rRows, rCols);
                if (h == IntPtr.Zero)
                    return "#ERR: " + FtpNative.GetLastError();

                try
                {
                    if (FtpNative.ftp_compute(h, method) != 0)
                        return "#ERR: " + FtpNative.GetLastError();

                    FtpNative.ftp_get_dims(h, out int rows, out int cols);
                    int size = rows * cols;
                    var buf = new double[size];

                    if (getter(h, buf, size) != 0)
                        return "#ERR: " + FtpNative.GetLastError();

                    return ToExcelArray(buf, rows, cols);
                }
                finally
                {
                    FtpNative.ftp_free(h);
                }
            }
            catch (Exception ex)
            {
                return "#ERR: " + ex.Message;
            }
        }

        /// <summary>
        /// Runs compute and returns all 7 matrices stacked vertically with label rows.
        /// </summary>
        private static object ComputeAll(object[,] outstanding, object[,] profiles, object[,] rates, int method)
        {
            try
            {
                var (outs, oRows) = FlattenColumn(outstanding);
                var (prof, pRows, pCols) = FlattenMatrix(profiles);
                var (rate, rRows, rCols) = FlattenMatrix(rates);

                IntPtr h = FtpNative.ftp_create(outs, oRows, prof, pRows, pCols, rate, rRows, rCols);
                if (h == IntPtr.Zero)
                    return "#ERR: " + FtpNative.GetLastError();

                try
                {
                    if (FtpNative.ftp_compute(h, method) != 0)
                        return "#ERR: " + FtpNative.GetLastError();

                    FtpNative.ftp_get_dims(h, out int rows, out int cols);
                    int size = rows * cols;

                    var labels = new[] {
                        "stock_amort", "stock_instal", "varstock_amort", "varstock_instal",
                        "ftp_rate", "ftp_int", "market_rate"
                    };
                    var getters = new GetterDelegate[] {
                        FtpNative.ftp_get_stock_amort,
                        FtpNative.ftp_get_stock_instal,
                        FtpNative.ftp_get_varstock_amort,
                        FtpNative.ftp_get_varstock_instal,
                        FtpNative.ftp_get_ftp_rate,
                        FtpNative.ftp_get_ftp_int,
                        FtpNative.ftp_get_market_rate
                    };

                    // Total rows = 7 * (1 label row + data rows)
                    int totalRows = 7 * (1 + rows);
                    var result = new object[totalRows, cols];

                    int outRow = 0;
                    for (int g = 0; g < 7; g++)
                    {
                        // Label row
                        result[outRow, 0] = labels[g];
                        for (int c = 1; c < cols; c++)
                            result[outRow, c] = "";
                        outRow++;

                        // Data
                        var buf = new double[size];
                        if (getters[g](h, buf, size) != 0)
                            return "#ERR: " + FtpNative.GetLastError();

                        for (int r = 0; r < rows; r++)
                        {
                            for (int c = 0; c < cols; c++)
                                result[outRow, c] = buf[r * cols + c];
                            outRow++;
                        }
                    }

                    return result;
                }
                finally
                {
                    FtpNative.ftp_free(h);
                }
            }
            catch (Exception ex)
            {
                return "#ERR: " + ex.Message;
            }
        }

        /// <summary>
        /// Converts an Excel object[,] column range to a flat double[].
        /// </summary>
        private static (double[] data, int rows) FlattenColumn(object[,] range)
        {
            int rows = range.GetLength(0);
            var data = new double[rows];
            for (int i = 0; i < rows; i++)
                data[i] = Convert.ToDouble(range[i, 0]);
            return (data, rows);
        }

        /// <summary>
        /// Converts an Excel object[,] range to a flat row-major double[].
        /// </summary>
        private static (double[] data, int rows, int cols) FlattenMatrix(object[,] range)
        {
            int rows = range.GetLength(0);
            int cols = range.GetLength(1);
            var data = new double[rows * cols];
            for (int r = 0; r < rows; r++)
                for (int c = 0; c < cols; c++)
                    data[r * cols + c] = Convert.ToDouble(range[r, c]);
            return (data, rows, cols);
        }

        /// <summary>
        /// Converts a flat row-major double[] to an Excel-friendly object[,].
        /// </summary>
        private static object[,] ToExcelArray(double[] buf, int rows, int cols)
        {
            var result = new object[rows, cols];
            for (int r = 0; r < rows; r++)
                for (int c = 0; c < cols; c++)
                    result[r, c] = buf[r * cols + c];
            return result;
        }
    }
}
