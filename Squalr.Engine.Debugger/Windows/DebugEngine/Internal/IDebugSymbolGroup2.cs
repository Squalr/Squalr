// Copyright (c) Microsoft. All rights reserved.
// Licensed under the MIT license. See LICENSE file in the project root for full license information.

using System;
using System.Text;
using System.Runtime.InteropServices;

#pragma warning disable 1591

namespace Microsoft.Diagnostics.Runtime.Interop
{
    [ComImport, InterfaceType(ComInterfaceType.InterfaceIsIUnknown), Guid("6a7ccc5f-fb5e-4dcc-b41c-6c20307bccc7")]
    public interface IDebugSymbolGroup2 : IDebugSymbolGroup
    {
        /* IDebugSymbolGroup */

        [PreserveSig]
        new Int32 GetNumberSymbols(
            [Out] out UInt32 Number);

        [PreserveSig]
        new Int32 AddSymbol(
            [In, MarshalAs(UnmanagedType.LPStr)] String Name,
            [In, Out] ref UInt32 Index);

        [PreserveSig]
        new Int32 RemoveSymbolByName(
            [In, MarshalAs(UnmanagedType.LPStr)] String Name);

        [PreserveSig]
        new Int32 RemoveSymbolsByIndex(
            [In] UInt32 Index);

        [PreserveSig]
        new Int32 GetSymbolName(
            [In] UInt32 Index,
            [Out, MarshalAs(UnmanagedType.LPStr)] StringBuilder Buffer,
            [In] Int32 BufferSize,
            [Out] out UInt32 NameSize);

        [PreserveSig]
        new Int32 GetSymbolParameters(
            [In] UInt32 Start,
            [In] UInt32 Count,
            [Out, MarshalAs(UnmanagedType.LPArray)] DEBUG_SYMBOL_PARAMETERS[] Params);

        [PreserveSig]
        new Int32 ExpandSymbol(
            [In] UInt32 Index,
            [In, MarshalAs(UnmanagedType.Bool)] Boolean Expand);

        [PreserveSig]
        new Int32 OutputSymbols(
            [In] DEBUG_OUTCTL OutputControl,
            [In] DEBUG_OUTPUT_SYMBOLS Flags,
            [In] UInt32 Start,
            [In] UInt32 Count);

        [PreserveSig]
        new Int32 WriteSymbol(
            [In] UInt32 Index,
            [In, MarshalAs(UnmanagedType.LPStr)] String Value);

        [PreserveSig]
        new Int32 OutputAsType(
            [In] UInt32 Index,
            [In, MarshalAs(UnmanagedType.LPStr)] String Type);

        /* IDebugSymbolGroup2 */

        [PreserveSig]
        Int32 AddSymbolWide(
            [In, MarshalAs(UnmanagedType.LPWStr)] String Name,
            [In, Out] ref UInt32 Index);

        [PreserveSig]
        Int32 RemoveSymbolByNameWide(
            [In, MarshalAs(UnmanagedType.LPWStr)] String Name);

        [PreserveSig]
        Int32 GetSymbolNameWide(
            [In] UInt32 Index,
            [Out, MarshalAs(UnmanagedType.LPWStr)] StringBuilder Buffer,
            [In] Int32 BufferSize,
            [Out] out UInt32 NameSize);

        [PreserveSig]
        Int32 WriteSymbolWide(
            [In] UInt32 Index,
            [In, MarshalAs(UnmanagedType.LPWStr)] String Value);

        [PreserveSig]
        Int32 OutputAsTypeWide(
            [In] UInt32 Index,
            [In, MarshalAs(UnmanagedType.LPWStr)] String Type);

        [PreserveSig]
        Int32 GetSymbolTypeName(
            [In] UInt32 Index,
            [Out, MarshalAs(UnmanagedType.LPStr)] StringBuilder Buffer,
            [In] Int32 BufferSize,
            [Out] out UInt32 NameSize);

        [PreserveSig]
        Int32 GetSymbolTypeNameWide(
            [In] UInt32 Index,
            [Out, MarshalAs(UnmanagedType.LPWStr)] StringBuilder Buffer,
            [In] Int32 BufferSize,
            [Out] out UInt32 NameSize);

        [PreserveSig]
        Int32 GetSymbolSize(
            [In] UInt32 Index,
            [Out] out UInt32 Size);

        [PreserveSig]
        Int32 GetSymbolOffset(
            [In] UInt32 Index,
            [Out] out UInt64 Offset);

        [PreserveSig]
        Int32 GetSymbolRegister(
            [In] UInt32 Index,
            [Out] out UInt32 Register);

        [PreserveSig]
        Int32 GetSymbolValueText(
            [In] UInt32 Index,
            [Out, MarshalAs(UnmanagedType.LPStr)] StringBuilder Buffer,
            [In] Int32 BufferSize,
            [Out] out UInt32 NameSize);

        [PreserveSig]
        Int32 GetSymbolValueTextWide(
            [In] UInt32 Index,
            [Out, MarshalAs(UnmanagedType.LPWStr)] StringBuilder Buffer,
            [In] Int32 BufferSize,
            [Out] out UInt32 NameSize);

        [PreserveSig]
        Int32 GetSymbolEntryInformation(
            [In] UInt32 Index,
            [Out] out DEBUG_SYMBOL_ENTRY Info);
    }
}