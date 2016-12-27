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
// Generated by IDLImporter from file nsIAutoCompletePopup.idl
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
    ///This Source Code Form is subject to the terms of the Mozilla Public
    /// License, v. 2.0. If a copy of the MPL was not distributed with this
    /// file, You can obtain one at http://mozilla.org/MPL/2.0/. </summary>
    [ComImport()]
	[InterfaceType(ComInterfaceType.InterfaceIsIUnknown)]
	[Guid("1b9d7d8a-6dd0-11dc-8314-0800200c9a66")]
	public interface nsIAutoCompletePopup
	{
		
		/// <summary>
        /// The input object that the popup is currently bound to
        /// </summary>
		[return: MarshalAs(UnmanagedType.Interface)]
		[MethodImpl(MethodImplOptions.InternalCall, MethodCodeType=MethodCodeType.Runtime)]
		nsIAutoCompleteInput GetInputAttribute();
		
		/// <summary>
        /// An alternative value to be used when text is entered, rather than the
        /// value of the selected item
        /// </summary>
		[MethodImpl(MethodImplOptions.InternalCall, MethodCodeType=MethodCodeType.Runtime)]
		void GetOverrideValueAttribute([MarshalAs(UnmanagedType.CustomMarshaler, MarshalType = "Gecko.CustomMarshalers.AStringMarshaler")] nsAStringBase aOverrideValue);
		
		/// <summary>
        /// The index of the result item that is currently selected
        /// </summary>
		[MethodImpl(MethodImplOptions.InternalCall, MethodCodeType=MethodCodeType.Runtime)]
		int GetSelectedIndexAttribute();
		
		/// <summary>
        /// The index of the result item that is currently selected
        /// </summary>
		[MethodImpl(MethodImplOptions.InternalCall, MethodCodeType=MethodCodeType.Runtime)]
		void SetSelectedIndexAttribute(int aSelectedIndex);
		
		/// <summary>
        /// Indicates if the popup is currently open
        /// </summary>
		[return: MarshalAs(UnmanagedType.U1)]
		[MethodImpl(MethodImplOptions.InternalCall, MethodCodeType=MethodCodeType.Runtime)]
		bool GetPopupOpenAttribute();
		
		/// <summary>
        /// Bind the popup to an input object and display it with the given coordinates
        ///
        /// @param input - The input object that the popup will be bound to
        /// @param element - The element that the popup will be aligned with
        /// </summary>
		[MethodImpl(MethodImplOptions.InternalCall, MethodCodeType=MethodCodeType.Runtime)]
		void OpenAutocompletePopup([MarshalAs(UnmanagedType.Interface)] nsIAutoCompleteInput input, [MarshalAs(UnmanagedType.Interface)] nsIDOMElement element);
		
		/// <summary>
        /// Close the popup and detach from the bound input
        /// </summary>
		[MethodImpl(MethodImplOptions.InternalCall, MethodCodeType=MethodCodeType.Runtime)]
		void ClosePopup();
		
		/// <summary>
        /// Instruct the result view to repaint itself to reflect the most current
        /// underlying data
        /// </summary>
		[MethodImpl(MethodImplOptions.InternalCall, MethodCodeType=MethodCodeType.Runtime)]
		void Invalidate();
		
		/// <summary>
        /// Change the selection relative to the current selection and make sure
        /// the newly selected row is visible
        ///
        /// @param reverse - Select a row above the current selection
        /// @param page - Select a row that is a full visible page from the current selection
        /// @return The currently selected result item index
        /// </summary>
		[MethodImpl(MethodImplOptions.InternalCall, MethodCodeType=MethodCodeType.Runtime)]
		void SelectBy([MarshalAs(UnmanagedType.U1)] bool reverse, [MarshalAs(UnmanagedType.U1)] bool page);
	}
}