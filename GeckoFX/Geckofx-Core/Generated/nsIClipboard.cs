// --------------------------------------------------------------------------------------------
// Version: MPL 1.1/GPL 2.0/LGPL 2.1
// 
// The contents of this file are subject to the Mozilla Public License Version
// 1.1 (the "License"); you may not use this file except in compliance with
// the License. You may obtain a copy of the License at
// http://www.mozilla.org/MPL/
// 
// Software distributed under the License is distributed on an "AS IS" basis,
// WITHOUT WARRANTY OF ANY KIND, either express or implied. See the License
// for the specific language governing rights and limitations under the
// License.
// 
// <remarks>
// Generated by IDLImporter from file nsIClipboard.idl
// 
// You should use these interfaces when you access the COM objects defined in the mentioned
// IDL/IDH file.
// </remarks>
// --------------------------------------------------------------------------------------------
namespace Gecko
{
    using System;
    using System.Runtime.CompilerServices;
    using System.Runtime.InteropServices;


    /// <summary>
    ///-*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*-
    ///
    /// This Source Code Form is subject to the terms of the Mozilla Public
    /// License, v. 2.0. If a copy of the MPL was not distributed with this
    /// file, You can obtain one at http://mozilla.org/MPL/2.0/. </summary>
    [ComImport()]
	[InterfaceType(ComInterfaceType.InterfaceIsIUnknown)]
	[Guid("ceaa0047-647f-4b8e-ad1c-aff9fa62aa51")]
	public interface nsIClipboard
	{
		
		/// <summary>
        /// Given a transferable, set the data on the native clipboard
        ///
        /// @param  aTransferable The transferable
        /// @param  anOwner The owner of the transferable
        /// @param  aWhichClipboard Specifies the clipboard to which this operation applies.
        /// @result NS_Ok if no errors
        /// </summary>
		[MethodImpl(MethodImplOptions.InternalCall, MethodCodeType=MethodCodeType.Runtime)]
		void SetData([MarshalAs(UnmanagedType.Interface)] nsITransferable aTransferable, [MarshalAs(UnmanagedType.Interface)] nsIClipboardOwner anOwner, int aWhichClipboard);
		
		/// <summary>
        /// Given a transferable, get the clipboard data.
        ///
        /// @param  aTransferable The transferable
        /// @param  aWhichClipboard Specifies the clipboard to which this operation applies.
        /// @result NS_Ok if no errors
        /// </summary>
		[MethodImpl(MethodImplOptions.InternalCall, MethodCodeType=MethodCodeType.Runtime)]
		void GetData([MarshalAs(UnmanagedType.Interface)] nsITransferable aTransferable, int aWhichClipboard);
		
		/// <summary>
        /// This empties the clipboard and notifies the clipboard owner.
        /// This empties the "logical" clipboard. It does not clear the native clipboard.
        ///
        /// @param  aWhichClipboard Specifies the clipboard to which this operation applies.
        /// @result NS_OK if successful.
        /// </summary>
		[MethodImpl(MethodImplOptions.InternalCall, MethodCodeType=MethodCodeType.Runtime)]
		void EmptyClipboard(int aWhichClipboard);
		
		/// <summary>
        /// This provides a way to give correct UI feedback about, for instance, a paste
        /// should be allowed. It does _NOT_ actually retreive the data and should be a very
        /// inexpensive call. All it does is check if there is data on the clipboard matching
        /// any of the flavors in the given list.
        ///
        /// @param  aFlavorList     An array of ASCII strings.
        /// @param  aLength         The length of the aFlavorList.
        /// @param  aWhichClipboard Specifies the clipboard to which this operation applies.
        /// @outResult - if data is present matching one of
        /// @result NS_OK if successful.
        /// </summary>
		[return: MarshalAs(UnmanagedType.U1)]
		[MethodImpl(MethodImplOptions.InternalCall, MethodCodeType=MethodCodeType.Runtime)]
		bool HasDataMatchingFlavors([MarshalAs(UnmanagedType.LPArray, SizeParamIndex=1)] string[] aFlavorList, uint aLength, int aWhichClipboard);
		
		/// <summary>
        /// Allows clients to determine if the implementation supports the concept of a
        /// separate clipboard for selection.
        ///
        /// @outResult - true if
        /// @result NS_OK if successful.
        /// </summary>
		[return: MarshalAs(UnmanagedType.U1)]
		[MethodImpl(MethodImplOptions.InternalCall, MethodCodeType=MethodCodeType.Runtime)]
		bool SupportsSelectionClipboard();
		
		/// <summary>
        /// Allows clients to determine if the implementation supports the concept of a
        /// separate clipboard for find search strings.
        ///
        /// @result NS_OK if successful.
        /// </summary>
		[return: MarshalAs(UnmanagedType.U1)]
		[MethodImpl(MethodImplOptions.InternalCall, MethodCodeType=MethodCodeType.Runtime)]
		bool SupportsFindClipboard();
	}
	
	/// <summary>nsIClipboardConsts </summary>
	public class nsIClipboardConsts
	{
		
		// <summary>
        //-*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*-
        //
        // This Source Code Form is subject to the terms of the Mozilla Public
        // License, v. 2.0. If a copy of the MPL was not distributed with this
        // file, You can obtain one at http://mozilla.org/MPL/2.0/. </summary>
		public const long kSelectionClipboard = 0;
		
		// 
		public const long kGlobalClipboard = 1;
		
		// 
		public const long kFindClipboard = 2;
	}
}