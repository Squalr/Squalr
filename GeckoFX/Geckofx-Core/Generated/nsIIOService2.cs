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
// Generated by IDLImporter from file nsIIOService2.idl
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
    /// nsIIOService2 extends nsIIOService
    /// </summary>
    [ComImport()]
	[InterfaceType(ComInterfaceType.InterfaceIsIUnknown)]
	[Guid("9a7dc724-0b5c-4b78-9722-1037074c02de")]
	public interface nsIIOService2 : nsIIOService
	{
		
		/// <summary>
        /// Returns a protocol handler for a given URI scheme.
        ///
        /// @param aScheme the URI scheme
        /// @return reference to corresponding nsIProtocolHandler
        /// </summary>
		[return: MarshalAs(UnmanagedType.Interface)]
		[MethodImpl(MethodImplOptions.InternalCall, MethodCodeType=MethodCodeType.Runtime)]
		new nsIProtocolHandler GetProtocolHandler([MarshalAs(UnmanagedType.LPStr)] string aScheme);
		
		/// <summary>
        /// Returns the protocol flags for a given scheme.
        ///
        /// @param aScheme the URI scheme
        /// @return value of corresponding nsIProtocolHandler::protocolFlags
        /// </summary>
		[MethodImpl(MethodImplOptions.InternalCall, MethodCodeType=MethodCodeType.Runtime)]
		new uint GetProtocolFlags([MarshalAs(UnmanagedType.LPStr)] string aScheme);
		
		/// <summary>
        /// This method constructs a new URI by determining the scheme of the
        /// URI spec, and then delegating the construction of the URI to the
        /// protocol handler for that scheme. QueryInterface can be used on
        /// the resulting URI object to obtain a more specific type of URI.
        ///
        /// @see nsIProtocolHandler::newURI
        /// </summary>
		[return: MarshalAs(UnmanagedType.Interface)]
		[MethodImpl(MethodImplOptions.InternalCall, MethodCodeType=MethodCodeType.Runtime)]
		new nsIURI NewURI([MarshalAs(UnmanagedType.LPStruct)] nsAUTF8StringBase aSpec, [MarshalAs(UnmanagedType.LPStr)] string aOriginCharset, [MarshalAs(UnmanagedType.Interface)] nsIURI aBaseURI);
		
		/// <summary>
        /// This method constructs a new URI from a nsIFile.
        ///
        /// @param aFile specifies the file path
        /// @return reference to a new nsIURI object
        ///
        /// Note: in the future, for perf reasons we should allow
        /// callers to specify whether this is a file or directory by
        /// splitting this  into newDirURI() and newActualFileURI().
        /// </summary>
		[return: MarshalAs(UnmanagedType.Interface)]
		[MethodImpl(MethodImplOptions.InternalCall, MethodCodeType=MethodCodeType.Runtime)]
		new nsIURI NewFileURI([MarshalAs(UnmanagedType.Interface)] nsIFile aFile);
		
		/// <summary>
        /// Creates a channel for a given URI.
        ///
        /// @param aURI nsIURI from which to make a channel
        /// @return reference to the new nsIChannel object
        /// </summary>
		[return: MarshalAs(UnmanagedType.Interface)]
		[MethodImpl(MethodImplOptions.InternalCall, MethodCodeType=MethodCodeType.Runtime)]
		new nsIChannel NewChannelFromURI([MarshalAs(UnmanagedType.Interface)] nsIURI aURI);
		
		/// <summary>
        /// Equivalent to newChannelFromURI(newURI(...))
        /// </summary>
		[return: MarshalAs(UnmanagedType.Interface)]
		[MethodImpl(MethodImplOptions.InternalCall, MethodCodeType=MethodCodeType.Runtime)]
		new nsIChannel NewChannel([MarshalAs(UnmanagedType.LPStruct)] nsAUTF8StringBase aSpec, [MarshalAs(UnmanagedType.LPStr)] string aOriginCharset, [MarshalAs(UnmanagedType.Interface)] nsIURI aBaseURI);
		
		/// <summary>
        /// Returns true if networking is in "offline" mode. When in offline mode,
        /// attempts to access the network will fail (although this does not
        /// necessarily correlate with whether there is actually a network
        /// available -- that's hard to detect without causing the dialer to
        /// come up).
        ///
        /// Changing this fires observer notifications ... see below.
        /// </summary>
		[return: MarshalAs(UnmanagedType.U1)]
		[MethodImpl(MethodImplOptions.InternalCall, MethodCodeType=MethodCodeType.Runtime)]
		new bool GetOfflineAttribute();
		
		/// <summary>
        /// Returns true if networking is in "offline" mode. When in offline mode,
        /// attempts to access the network will fail (although this does not
        /// necessarily correlate with whether there is actually a network
        /// available -- that's hard to detect without causing the dialer to
        /// come up).
        ///
        /// Changing this fires observer notifications ... see below.
        /// </summary>
		[MethodImpl(MethodImplOptions.InternalCall, MethodCodeType=MethodCodeType.Runtime)]
		new void SetOfflineAttribute([MarshalAs(UnmanagedType.U1)] bool aOffline);
		
		/// <summary>
        /// Checks if a port number is banned. This involves consulting a list of
        /// unsafe ports, corresponding to network services that may be easily
        /// exploitable. If the given port is considered unsafe, then the protocol
        /// handler (corresponding to aScheme) will be asked whether it wishes to
        /// override the IO service's decision to block the port. This gives the
        /// protocol handler ultimate control over its own security policy while
        /// ensuring reasonable, default protection.
        ///
        /// @see nsIProtocolHandler::allowPort
        /// </summary>
		[return: MarshalAs(UnmanagedType.U1)]
		[MethodImpl(MethodImplOptions.InternalCall, MethodCodeType=MethodCodeType.Runtime)]
		new bool AllowPort(int aPort, [MarshalAs(UnmanagedType.LPStr)] string aScheme);
		
		/// <summary>
        /// Utility to extract the scheme from a URL string, consistently and
        /// according to spec (see RFC 2396).
        ///
        /// NOTE: Most URL parsing is done via nsIURI, and in fact the scheme
        /// can also be extracted from a URL string via nsIURI.  This method
        /// is provided purely as an optimization.
        ///
        /// @param aSpec the URL string to parse
        /// @return URL scheme
        ///
        /// @throws NS_ERROR_MALFORMED_URI if URL string is not of the right form.
        /// </summary>
		[MethodImpl(MethodImplOptions.InternalCall, MethodCodeType=MethodCodeType.Runtime)]
		new void ExtractScheme([MarshalAs(UnmanagedType.LPStruct)] nsAUTF8StringBase urlString, [MarshalAs(UnmanagedType.LPStruct)] nsACStringBase retval);
		
		/// <summary>
        /// While this is set, IOService will monitor an nsINetworkLinkService
        /// (if available) and set its offline status to "true" whenever
        /// isLinkUp is false.
        ///
        /// Applications that want to control changes to the IOService's offline
        /// status should set this to false, watch for network:link-status-changed
        /// broadcasts, and change nsIIOService::offline as they see fit. Note
        /// that this means during application startup, IOService may be offline
        /// if there is no link, until application code runs and can turn off
        /// this management.
        /// </summary>
		[return: MarshalAs(UnmanagedType.U1)]
		[MethodImpl(MethodImplOptions.InternalCall, MethodCodeType=MethodCodeType.Runtime)]
		bool GetManageOfflineStatusAttribute();
		
		/// <summary>
        /// While this is set, IOService will monitor an nsINetworkLinkService
        /// (if available) and set its offline status to "true" whenever
        /// isLinkUp is false.
        ///
        /// Applications that want to control changes to the IOService's offline
        /// status should set this to false, watch for network:link-status-changed
        /// broadcasts, and change nsIIOService::offline as they see fit. Note
        /// that this means during application startup, IOService may be offline
        /// if there is no link, until application code runs and can turn off
        /// this management.
        /// </summary>
		[MethodImpl(MethodImplOptions.InternalCall, MethodCodeType=MethodCodeType.Runtime)]
		void SetManageOfflineStatusAttribute([MarshalAs(UnmanagedType.U1)] bool aManageOfflineStatus);
		
		/// <summary>
        /// Creates a channel for a given URI.
        ///
        /// @param aURI nsIURI from which to make a channel
        /// @param aProxyURI nsIURI to use for proxy resolution. Can be null in which
        /// case aURI is used
        /// @param aProxyFlags flags from nsIProtocolProxyService to use
        /// when resolving proxies for this new channel
        /// @return reference to the new nsIChannel object
        /// </summary>
		[return: MarshalAs(UnmanagedType.Interface)]
		[MethodImpl(MethodImplOptions.InternalCall, MethodCodeType=MethodCodeType.Runtime)]
		nsIChannel NewChannelFromURIWithProxyFlags([MarshalAs(UnmanagedType.Interface)] nsIURI aURI, [MarshalAs(UnmanagedType.Interface)] nsIURI aProxyURI, uint aProxyFlags);
	}
}