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
// Generated by IDLImporter from file nsIDOMCSSRuleList.idl
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
    /// The nsIDOMCSSRuleList interface is a datatype for a list of CSS
    /// style rules in the Document Object Model.
    ///
    /// For more information on this interface please see
    /// http://www.w3.org/TR/DOM-Level-2-Style
    /// </summary>
    [ComImport()]
	[InterfaceType(ComInterfaceType.InterfaceIsIUnknown)]
	[Guid("a6cf90c0-15b3-11d2-932e-00805f8add32")]
	public interface nsIDOMCSSRuleList
	{
		
		/// <summary>
        /// The nsIDOMCSSRuleList interface is a datatype for a list of CSS
        /// style rules in the Document Object Model.
        ///
        /// For more information on this interface please see
        /// http://www.w3.org/TR/DOM-Level-2-Style
        /// </summary>
		[MethodImpl(MethodImplOptions.InternalCall, MethodCodeType=MethodCodeType.Runtime)]
		uint GetLengthAttribute();
		
		[return: MarshalAs(UnmanagedType.Interface)]
		[MethodImpl(MethodImplOptions.InternalCall, MethodCodeType=MethodCodeType.Runtime)]
		nsIDOMCSSRule Item(uint index);
	}
}