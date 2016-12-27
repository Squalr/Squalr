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
// Generated by IDLImporter from file nsIURIRefObject.idl
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
    ///A class which can represent any node which points to an
    /// external URI, e.g. <a>, <img>, <script> etc,
    /// and has the capability to rewrite URLs to be
    /// relative or absolute.
    /// Used by the editor but not dependant on it.
    /// </summary>
    [ComImport()]
	[InterfaceType(ComInterfaceType.InterfaceIsIUnknown)]
	[Guid("2226927e-1dd2-11b2-b57f-faab47288563")]
	public interface nsIURIRefObject
	{
		
		/// <summary>
        ///A class which can represent any node which points to an
        /// external URI, e.g. <a>, <img>, <script> etc,
        /// and has the capability to rewrite URLs to be
        /// relative or absolute.
        /// Used by the editor but not dependant on it.
        /// </summary>
		[return: MarshalAs(UnmanagedType.Interface)]
		[MethodImpl(MethodImplOptions.InternalCall, MethodCodeType=MethodCodeType.Runtime)]
		nsIDOMNode GetNodeAttribute();
		
		/// <summary>
        ///A class which can represent any node which points to an
        /// external URI, e.g. <a>, <img>, <script> etc,
        /// and has the capability to rewrite URLs to be
        /// relative or absolute.
        /// Used by the editor but not dependant on it.
        /// </summary>
		[MethodImpl(MethodImplOptions.InternalCall, MethodCodeType=MethodCodeType.Runtime)]
		void SetNodeAttribute([MarshalAs(UnmanagedType.Interface)] nsIDOMNode aNode);
		
		/// <summary>
        /// Go back to the beginning of the attribute list.
        /// </summary>
		[MethodImpl(MethodImplOptions.InternalCall, MethodCodeType=MethodCodeType.Runtime)]
		void Reset();
		
		/// <summary>
        /// Return the next rewritable URI.
        /// </summary>
		[MethodImpl(MethodImplOptions.InternalCall, MethodCodeType=MethodCodeType.Runtime)]
		void GetNextURI([MarshalAs(UnmanagedType.CustomMarshaler, MarshalType = "Gecko.CustomMarshalers.AStringMarshaler")] nsAStringBase retval);
		
		/// <summary>
        /// Go back to the beginning of the attribute list
        ///
        /// @param aOldPat  Old pattern to be replaced, e.g. file:///a/b/
        /// @param aNewPat  New pattern to be replaced, e.g. http://mypage.aol.com/
        /// @param aMakeRel Rewrite links as relative vs. absolute
        /// </summary>
		[MethodImpl(MethodImplOptions.InternalCall, MethodCodeType=MethodCodeType.Runtime)]
		void RewriteAllURIs([MarshalAs(UnmanagedType.CustomMarshaler, MarshalType = "Gecko.CustomMarshalers.AStringMarshaler")] nsAStringBase aOldPat, [MarshalAs(UnmanagedType.CustomMarshaler, MarshalType = "Gecko.CustomMarshalers.AStringMarshaler")] nsAStringBase aNewPat, [MarshalAs(UnmanagedType.U1)] bool aMakeRel);
	}
}