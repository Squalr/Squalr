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
// Generated by IDLImporter from file nsIProtocolProxyCallback.idl
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
    /// This interface serves as a closure for nsIProtocolProxyService's
    /// asyncResolve method.
    /// </summary>
    [ComImport()]
	[InterfaceType(ComInterfaceType.InterfaceIsIUnknown)]
	[Guid("a9967200-f95e-45c2-beb3-9b060d874bfd")]
	public interface nsIProtocolProxyCallback
	{
		
		/// <summary>
        /// This method is called when proxy info is available or when an error
        /// in the proxy resolution occurs.
        ///
        /// @param aRequest
        /// The value returned from asyncResolve.
        /// @param aURI
        /// The URI passed to asyncResolve.
        /// @param aProxyInfo
        /// The resulting proxy info or null if there is no associated proxy
        /// info for aURI.  As with the result of nsIProtocolProxyService's
        /// resolve method, a null result implies that a direct connection
        /// should be used.
        /// @param aStatus
        /// The status of the callback.  This is a failure code if the request
        /// could not be satisfied, in which case the value of aStatus
        /// indicates the reason for the failure and aProxyInfo will be null.
        /// </summary>
		[MethodImpl(MethodImplOptions.InternalCall, MethodCodeType=MethodCodeType.Runtime)]
		void OnProxyAvailable([MarshalAs(UnmanagedType.Interface)] nsICancelable aRequest, [MarshalAs(UnmanagedType.Interface)] nsIURI aURI, [MarshalAs(UnmanagedType.Interface)] nsIProxyInfo aProxyInfo, int aStatus);
	}
}