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
// Generated by IDLImporter from file nsIAccessibleHyperLink.idl
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
    /// A cross-platform interface that supports hyperlink-specific properties and
    /// methods.  Anchors, image maps, xul:labels with class="text-link" implement this interface.
    /// </summary>
    [ComImport()]
	[InterfaceType(ComInterfaceType.InterfaceIsIUnknown)]
	[Guid("883643d4-93a5-4f32-922c-6f06e01363c1")]
	public interface nsIAccessibleHyperLink
	{
		
		/// <summary>
        /// Returns the offset of the link within the parent accessible.
        /// </summary>
		[MethodImpl(MethodImplOptions.InternalCall, MethodCodeType=MethodCodeType.Runtime)]
		int GetStartIndexAttribute();
		
		/// <summary>
        /// Returns the end index of the link within the parent accessible.
        ///
        /// @note  The link itself is represented by one embedded character within the
        /// parent text, so the endIndex should be startIndex + 1.
        /// </summary>
		[MethodImpl(MethodImplOptions.InternalCall, MethodCodeType=MethodCodeType.Runtime)]
		int GetEndIndexAttribute();
		
		/// <summary>
        /// Determines whether the link is valid (e. g. points to a valid URL).
        ///
        /// @note  XXX Currently only used with ARIA links, and the author has to
        /// specify that the link is invalid via the aria-invalid="true" attribute.
        /// In all other cases, TRUE is returned.
        /// </summary>
		[return: MarshalAs(UnmanagedType.U1)]
		[MethodImpl(MethodImplOptions.InternalCall, MethodCodeType=MethodCodeType.Runtime)]
		bool GetValidAttribute();
		
		/// <summary>
        /// The numbber of anchors within this Hyperlink. Is normally 1 for anchors.
        /// This anchor is, for example, the visible output of the html:a tag.
        /// With an Image Map, reflects the actual areas within the map.
        /// </summary>
		[MethodImpl(MethodImplOptions.InternalCall, MethodCodeType=MethodCodeType.Runtime)]
		int GetAnchorCountAttribute();
		
		/// <summary>
        /// Returns the URI at the given index.
        ///
        /// @note  ARIA hyperlinks do not have an URI to point to, since clicks are
        /// processed via JavaScript. Therefore this property does not work on ARIA
        /// links.
        ///
        /// @param index  The 0-based index of the URI to be returned.
        ///
        /// @return the nsIURI object containing the specifications for the URI.
        /// </summary>
		[return: MarshalAs(UnmanagedType.Interface)]
		[MethodImpl(MethodImplOptions.InternalCall, MethodCodeType=MethodCodeType.Runtime)]
		nsIURI GetURI(int index);
		
		/// <summary>
        /// Returns a reference to the object at the given index.
        ///
        /// @param index  The 0-based index whose object is to be returned.
        ///
        /// @return the nsIAccessible object at the desired index.
        /// </summary>
		[return: MarshalAs(UnmanagedType.Interface)]
		[MethodImpl(MethodImplOptions.InternalCall, MethodCodeType=MethodCodeType.Runtime)]
		nsIAccessible GetAnchor(int index);
	}
}
