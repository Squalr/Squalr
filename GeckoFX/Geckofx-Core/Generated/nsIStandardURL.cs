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
// Generated by IDLImporter from file nsIStandardURL.idl
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
    /// nsIStandardURL defines the interface to an URL with the standard
    /// file path format common to protocols like http, ftp, and file.
    /// It supports initialization from a relative path and provides
    /// some customization on how URLs are normalized.
    /// </summary>
    [ComImport()]
	[InterfaceType(ComInterfaceType.InterfaceIsIUnknown)]
	[Guid("babd6cca-ebe7-4329-967c-d6b9e33caa81")]
	public interface nsIStandardURL : nsIMutable
	{
		
		/// <summary>
        /// Control whether or not this object can be modified.  If the flag is
        /// false, no modification is allowed.  Once the flag has been set to false,
        /// it cannot be reset back to true -- attempts to do so throw
        /// NS_ERROR_INVALID_ARG.
        /// </summary>
		[return: MarshalAs(UnmanagedType.U1)]
		[MethodImpl(MethodImplOptions.InternalCall, MethodCodeType=MethodCodeType.Runtime)]
		new bool GetMutableAttribute();
		
		/// <summary>
        /// Control whether or not this object can be modified.  If the flag is
        /// false, no modification is allowed.  Once the flag has been set to false,
        /// it cannot be reset back to true -- attempts to do so throw
        /// NS_ERROR_INVALID_ARG.
        /// </summary>
		[MethodImpl(MethodImplOptions.InternalCall, MethodCodeType=MethodCodeType.Runtime)]
		new void SetMutableAttribute([MarshalAs(UnmanagedType.U1)] bool aMutable);
		
		/// <summary>
        /// Initialize a standard URL.
        ///
        /// @param aUrlType       - one of the URLTYPE_ flags listed above.
        /// @param aDefaultPort   - if the port parsed from the URL string matches
        /// this port, then the port will be removed from the
        /// canonical form of the URL.
        /// @param aSpec          - URL string.
        /// @param aOriginCharset - the charset from which this URI string
        /// originated.  this corresponds to the charset
        /// that should be used when communicating this
        /// URI to an origin server, for example.  if
        /// null, then provide aBaseURI implements this
        /// interface, the origin charset of aBaseURI will
        /// be assumed, otherwise defaulting to UTF-8 (i.e.,
        /// no charset transformation from aSpec).
        /// @param aBaseURI       - if null, aSpec must specify an absolute URI.
        /// otherwise, aSpec will be resolved relative
        /// to aBaseURI.
        /// </summary>
		[MethodImpl(MethodImplOptions.InternalCall, MethodCodeType=MethodCodeType.Runtime)]
		void Init(uint aUrlType, int aDefaultPort, [MarshalAs(UnmanagedType.LPStruct)] nsAUTF8StringBase aSpec, [MarshalAs(UnmanagedType.LPStr)] string aOriginCharset, [MarshalAs(UnmanagedType.Interface)] nsIURI aBaseURI);
	}
	
	/// <summary>nsIStandardURLConsts </summary>
	public class nsIStandardURLConsts
	{
		
		// <summary>
        // blah:foo/bar    => blah://foo/bar
        // blah:/foo/bar   => blah:///foo/bar
        // blah://foo/bar  => blah://foo/bar
        // blah:///foo/bar => blah:///foo/bar
        // </summary>
		public const ulong URLTYPE_STANDARD = 1;
		
		// <summary>
        // blah:foo/bar    => blah://foo/bar
        // blah:/foo/bar   => blah://foo/bar
        // blah://foo/bar  => blah://foo/bar
        // blah:///foo/bar => blah://foo/bar
        // </summary>
		public const ulong URLTYPE_AUTHORITY = 2;
		
		// <summary>
        // blah:foo/bar    => blah:///foo/bar
        // blah:/foo/bar   => blah:///foo/bar
        // blah://foo/bar  => blah://foo/bar
        // blah:///foo/bar => blah:///foo/bar
        // </summary>
		public const ulong URLTYPE_NO_AUTHORITY = 3;
	}
}