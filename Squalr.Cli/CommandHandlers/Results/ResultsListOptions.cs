﻿namespace Squalr.Cli.CommandHandlers.Results
{
    using CommandLine;
    using Squalr.Engine.Common;
    using Squalr.Engine.Scanning;
    using Squalr.Engine.Scanning.Snapshots;
    using System;

    [Verb("list", HelpText = "List scan results.")]
    public class ResultsListOptions
    {
        private const UInt64 PageSize = 16;

        public Int32 Handle()
        {
            Snapshot results = SessionManager.Session.SnapshotManager.GetActiveSnapshot();

            if (results == null)
            {
                Console.WriteLine("[Error] No active scan results.");

                return -1;
            }

            UInt64 pageStart = this.Page * ResultsListOptions.PageSize;
            UInt64 pageEnd = Math.Min(pageStart + ResultsListOptions.PageSize, results.ElementCount);
            UInt64 pageCount = results.ElementCount / ResultsListOptions.PageSize;

            Console.WriteLine("----------------------------------------------");
            Console.WriteLine("Results for page " + this.Page + " / " + pageCount);
            Console.WriteLine("# " + "\t|\t" + "Address" + "\t|\t" + "Value");
            Console.WriteLine("----------------------------------------------");

            for (UInt64 index = pageStart; index < pageEnd; index++)
            {
                MemoryAlignment alignment = ScanSettings.ResolveAutoAlignment(ScanSettings.Alignment, ScanSettings.DataType.Size);
                Object currentValue = results[index, ScanSettings.Alignment].LoadCurrentValue(ScanSettings.DataType);
                String str;

                switch (currentValue)
                {
                    case Single fVal:
                        str = fVal.ToString(".0###########f");
                        break;
                    default:
                        str = currentValue.ToString();
                        break;
                }

                Console.WriteLine(index + "\t|\t" + Conversions.ToHex<UInt64>(results[index, alignment].GetBaseAddress()) + "\t|\t" + str);
            }

            Console.WriteLine();

            return 0;
        }

        [Option('p', "page", Required = false, HelpText = "Specifies the page of results to list")]
        public UInt64 Page { get; private set; }
    }
    //// End class
}
//// End namespace
