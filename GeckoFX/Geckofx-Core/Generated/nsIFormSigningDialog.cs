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
// Generated by IDLImporter from file nsIFormSigningDialog.idl
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
    /// nsIFormSigningDialog
    /// Provides UI for form signing.
    /// </summary>
    [ComImport()]
	[InterfaceType(ComInterfaceType.InterfaceIsIUnknown)]
	[Guid("4fe04d6d-4b66-4023-a0bc-b43ce68b3e15")]
	public interface nsIFormSigningDialog
	{
		
		/// <summary>
        /// confirmSignText
        /// UI shown when a web site calls crypto.signText,
        /// asking the user to confirm the confirm the signing request.
        ///
        /// returns true if the user confirmed, false on cancel
        /// </summary>
		[return: MarshalAs(UnmanagedType.U1)]
		[MethodImpl(MethodImplOptions.InternalCall, MethodCodeType=MethodCodeType.Runtime)]
		bool ConfirmSignText([MarshalAs(UnmanagedType.Interface)] nsIInterfaceRequestor ctxt, [MarshalAs(UnmanagedType.CustomMarshaler, MarshalType = "Gecko.CustomMarshalers.AStringMarshaler")] nsAStringBase host, [MarshalAs(UnmanagedType.CustomMarshaler, MarshalType = "Gecko.CustomMarshalers.AStringMarshaler")] nsAStringBase signText, [MarshalAs(UnmanagedType.LPArray, SizeParamIndex=5)] System.IntPtr[] certNickList, [MarshalAs(UnmanagedType.LPArray, SizeParamIndex=5)] System.IntPtr[] certDetailsList, uint count, ref int selectedIndex, [MarshalAs(UnmanagedType.CustomMarshaler, MarshalType = "Gecko.CustomMarshalers.AStringMarshaler")] nsAStringBase password);
	}
}