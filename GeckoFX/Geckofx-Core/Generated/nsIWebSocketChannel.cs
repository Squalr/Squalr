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
// Generated by IDLImporter from file nsIWebSocketChannel.idl
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
    /// Low-level websocket API: handles network protocol.
    ///
    /// This is primarly intended for use by the higher-level nsIWebSocket.idl.
    /// We are also making it scriptable for now, but this may change once we have
    /// WebSockets for Workers.
    /// </summary>
    [ComImport()]
	[InterfaceType(ComInterfaceType.InterfaceIsIUnknown)]
	[Guid("9ee5874c-ec39-4bc2-b2d7-194a4c98c9d2")]
	public interface nsIWebSocketChannel
	{
		
		/// <summary>
        /// The original URI used to construct the protocol connection. This is used
        /// in the case of a redirect or URI "resolution" (e.g. resolving a
        /// resource: URI to a file: URI) so that the original pre-redirect
        /// URI can still be obtained.  This is never null.
        /// </summary>
		[return: MarshalAs(UnmanagedType.Interface)]
		[MethodImpl(MethodImplOptions.InternalCall, MethodCodeType=MethodCodeType.Runtime)]
		nsIURI GetOriginalURIAttribute();
		
		/// <summary>
        /// The readonly URI corresponding to the protocol connection after any
        /// redirections are completed.
        /// </summary>
		[return: MarshalAs(UnmanagedType.Interface)]
		[MethodImpl(MethodImplOptions.InternalCall, MethodCodeType=MethodCodeType.Runtime)]
		nsIURI GetURIAttribute();
		
		/// <summary>
        /// The notification callbacks for authorization, etc..
        /// </summary>
		[return: MarshalAs(UnmanagedType.Interface)]
		[MethodImpl(MethodImplOptions.InternalCall, MethodCodeType=MethodCodeType.Runtime)]
		nsIInterfaceRequestor GetNotificationCallbacksAttribute();
		
		/// <summary>
        /// The notification callbacks for authorization, etc..
        /// </summary>
		[MethodImpl(MethodImplOptions.InternalCall, MethodCodeType=MethodCodeType.Runtime)]
		void SetNotificationCallbacksAttribute([MarshalAs(UnmanagedType.Interface)] nsIInterfaceRequestor aNotificationCallbacks);
		
		/// <summary>
        /// Transport-level security information (if any)
        /// </summary>
		[return: MarshalAs(UnmanagedType.Interface)]
		[MethodImpl(MethodImplOptions.InternalCall, MethodCodeType=MethodCodeType.Runtime)]
		nsISupports GetSecurityInfoAttribute();
		
		/// <summary>
        /// The load group of the websockets code.
        /// </summary>
		[return: MarshalAs(UnmanagedType.Interface)]
		[MethodImpl(MethodImplOptions.InternalCall, MethodCodeType=MethodCodeType.Runtime)]
		nsILoadGroup GetLoadGroupAttribute();
		
		/// <summary>
        /// The load group of the websockets code.
        /// </summary>
		[MethodImpl(MethodImplOptions.InternalCall, MethodCodeType=MethodCodeType.Runtime)]
		void SetLoadGroupAttribute([MarshalAs(UnmanagedType.Interface)] nsILoadGroup aLoadGroup);
		
		/// <summary>
        /// Sec-Websocket-Protocol value
        /// </summary>
		[MethodImpl(MethodImplOptions.InternalCall, MethodCodeType=MethodCodeType.Runtime)]
		void GetProtocolAttribute([MarshalAs(UnmanagedType.LPStruct)] nsACStringBase aProtocol);
		
		/// <summary>
        /// Sec-Websocket-Protocol value
        /// </summary>
		[MethodImpl(MethodImplOptions.InternalCall, MethodCodeType=MethodCodeType.Runtime)]
		void SetProtocolAttribute([MarshalAs(UnmanagedType.LPStruct)] nsACStringBase aProtocol);
		
		/// <summary>
        /// Sec-Websocket-Extensions response header value
        /// </summary>
		[MethodImpl(MethodImplOptions.InternalCall, MethodCodeType=MethodCodeType.Runtime)]
		void GetExtensionsAttribute([MarshalAs(UnmanagedType.LPStruct)] nsACStringBase aExtensions);
		
		/// <summary>
        /// Asynchronously open the websocket connection.  Received messages are fed
        /// to the socket listener as they arrive.  The socket listener's methods
        /// are called on the thread that calls asyncOpen and are not called until
        /// after asyncOpen returns.  If asyncOpen returns successfully, the
        /// protocol implementation promises to call at least onStop on the listener.
        ///
        /// NOTE: Implementations should throw NS_ERROR_ALREADY_OPENED if the
        /// websocket connection is reopened.
        ///
        /// @param aURI the uri of the websocket protocol - may be redirected
        /// @param aOrigin the uri of the originating resource
        /// @param aListener the nsIWebSocketListener implementation
        /// @param aContext an opaque parameter forwarded to aListener's methods
        /// </summary>
		[MethodImpl(MethodImplOptions.InternalCall, MethodCodeType=MethodCodeType.Runtime)]
		void AsyncOpen([MarshalAs(UnmanagedType.Interface)] nsIURI aURI, [MarshalAs(UnmanagedType.LPStruct)] nsACStringBase aOrigin, [MarshalAs(UnmanagedType.Interface)] nsIWebSocketListener aListener, [MarshalAs(UnmanagedType.Interface)] nsISupports aContext);
		
		/// <summary>
        /// Close the websocket connection for writing - no more calls to sendMsg
        /// or sendBinaryMsg should be made after calling this. The listener object
        /// may receive more messages if a server close has not yet been received.
        ///
        /// @param aCode the websocket closing handshake close code. Set to 0 if
        /// you are not providing a code.
        /// @param aReason the websocket closing handshake close reason
        /// </summary>
		[MethodImpl(MethodImplOptions.InternalCall, MethodCodeType=MethodCodeType.Runtime)]
		void Close(ushort aCode, [MarshalAs(UnmanagedType.LPStruct)] nsAUTF8StringBase aReason);
		
		/// <summary>
        /// Use to send text message down the connection to WebSocket peer.
        ///
        /// @param aMsg the utf8 string to send
        /// </summary>
		[MethodImpl(MethodImplOptions.InternalCall, MethodCodeType=MethodCodeType.Runtime)]
		void SendMsg([MarshalAs(UnmanagedType.LPStruct)] nsAUTF8StringBase aMsg);
		
		/// <summary>
        /// Use to send binary message down the connection to WebSocket peer.
        ///
        /// @param aMsg the data to send
        /// </summary>
		[MethodImpl(MethodImplOptions.InternalCall, MethodCodeType=MethodCodeType.Runtime)]
		void SendBinaryMsg([MarshalAs(UnmanagedType.LPStruct)] nsACStringBase aMsg);
		
		/// <summary>
        /// Use to send a binary stream (Blob) to Websocket peer.
        ///
        /// @param aStream The input stream to be sent.
        /// </summary>
		[MethodImpl(MethodImplOptions.InternalCall, MethodCodeType=MethodCodeType.Runtime)]
		void SendBinaryStream([MarshalAs(UnmanagedType.Interface)] nsIInputStream aStream, uint length);
		
		/// <summary>
        /// This value determines how often (in seconds) websocket keepalive
        /// pings are sent.  If set to 0 (the default), no pings are ever sent.
        ///
        /// This value can currently only be set before asyncOpen is called, else
        /// NS_ERROR_IN_PROGRESS is thrown.
        ///
        /// Be careful using this setting: ping traffic can consume lots of power and
        /// bandwidth over time.
        /// </summary>
		[MethodImpl(MethodImplOptions.InternalCall, MethodCodeType=MethodCodeType.Runtime)]
		uint GetPingIntervalAttribute();
		
		/// <summary>
        /// This value determines how often (in seconds) websocket keepalive
        /// pings are sent.  If set to 0 (the default), no pings are ever sent.
        ///
        /// This value can currently only be set before asyncOpen is called, else
        /// NS_ERROR_IN_PROGRESS is thrown.
        ///
        /// Be careful using this setting: ping traffic can consume lots of power and
        /// bandwidth over time.
        /// </summary>
		[MethodImpl(MethodImplOptions.InternalCall, MethodCodeType=MethodCodeType.Runtime)]
		void SetPingIntervalAttribute(uint aPingInterval);
		
		/// <summary>
        /// This value determines how long (in seconds) the websocket waits for
        /// the server to reply to a ping that has been sent before considering the
        /// connection broken.
        ///
        /// This value can currently only be set before asyncOpen is called, else
        /// NS_ERROR_IN_PROGRESS is thrown.
        /// </summary>
		[MethodImpl(MethodImplOptions.InternalCall, MethodCodeType=MethodCodeType.Runtime)]
		uint GetPingTimeoutAttribute();
		
		/// <summary>
        /// This value determines how long (in seconds) the websocket waits for
        /// the server to reply to a ping that has been sent before considering the
        /// connection broken.
        ///
        /// This value can currently only be set before asyncOpen is called, else
        /// NS_ERROR_IN_PROGRESS is thrown.
        /// </summary>
		[MethodImpl(MethodImplOptions.InternalCall, MethodCodeType=MethodCodeType.Runtime)]
		void SetPingTimeoutAttribute(uint aPingTimeout);
	}
	
	/// <summary>nsIWebSocketChannelConsts </summary>
	public class nsIWebSocketChannelConsts
	{
		
		// <summary>
        // section 7.4.1 defines these close codes
        // </summary>
		public const ushort CLOSE_NORMAL = 1000;
		
		// 
		public const ushort CLOSE_GOING_AWAY = 1001;
		
		// 
		public const ushort CLOSE_PROTOCOL_ERROR = 1002;
		
		// 
		public const ushort CLOSE_UNSUPPORTED_DATATYPE = 1003;
		
		// <summary>
        //  code 1004 is reserved
        // </summary>
		public const ushort CLOSE_NO_STATUS = 1005;
		
		// 
		public const ushort CLOSE_ABNORMAL = 1006;
		
		// 
		public const ushort CLOSE_INVALID_PAYLOAD = 1007;
		
		// 
		public const ushort CLOSE_POLICY_VIOLATION = 1008;
		
		// 
		public const ushort CLOSE_TOO_LARGE = 1009;
		
		// 
		public const ushort CLOSE_EXTENSION_MISSING = 1010;
		
		// <summary>
        // http://www.ietf.org/mail-archive/web/hybi/current/msg09372.html
        // </summary>
		public const ushort CLOSE_INTERNAL_ERROR = 1011;
		
		// <summary>
        // To be used if TLS handshake failed (ex: server certificate unverifiable)
        // </summary>
		public const ushort CLOSE_TLS_FAILED = 1015;
	}
}