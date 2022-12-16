// Copyright (c) Microsoft. All rights reserved.
// Licensed under the MIT license. See LICENSE file in the project root for full license information.

using System;
using System.Text;
using System.Runtime.InteropServices;

#pragma warning disable 1591

namespace Microsoft.Diagnostics.Runtime.Interop
{
    [ComImport, InterfaceType(ComInterfaceType.InterfaceIsIUnknown), Guid("f2528316-0f1a-4431-aeed-11d096e1e2ab")]
    public interface IDebugSymbolGroup
    {
        /* IDebugSymbolGroup */

        [PreserveSig]
        Int32 GetNumberSymbols(
            [Out] out UInt32 Number);

        [PreserveSig]
        Int32 AddSymbol(
            [In, MarshalAs(UnmanagedType.LPStr)] String Name,
            [In, Out] ref UInt32 Index);

        [PreserveSig]
        Int32 RemoveSymbolByName(
            [In, MarshalAs(UnmanagedType.LPStr)] String Name);

        [PreserveSig]
        Int32 RemoveSymbolsByIndex(
            [In] UInt32 Index);

        [PreserveSig]
        Int32 GetSymbolName(
            [In] UInt32 Index,
            [Out, MarshalAs(UnmanagedType.LPStr)] StringBuilder Buffer,
            [In] Int32 BufferSize,
            [Out] out UInt32 NameSize);

        [PreserveSig]
        Int32 GetSymbolParameters(
            [In] UInt32 Start,
            [In] UInt32 Count,
            [Out, MarshalAs(UnmanagedType.LPArray)] DEBUG_SYMBOL_PARAMETERS[] Params);

        [PreserveSig]
        Int32 ExpandSymbol(
            [In] UInt32 Index,
            [In, MarshalAs(UnmanagedType.Bool)] Boolean Expand);

        [PreserveSig]
        Int32 OutputSymbols(
            [In] DEBUG_OUTCTL OutputControl,
            [In] DEBUG_OUTPUT_SYMBOLS Flags,
            [In] UInt32 Start,
            [In] UInt32 Count);

        [PreserveSig]
        Int32 WriteSymbol(
            [In] UInt32 Index,
            [In, MarshalAs(UnmanagedType.LPStr)] String Value);

        [PreserveSig]
        Int32 OutputAsType(
            [In] UInt32 Index,
            [In, MarshalAs(UnmanagedType.LPStr)] String Type);
    }
    //// End interface
}
//// End namespace