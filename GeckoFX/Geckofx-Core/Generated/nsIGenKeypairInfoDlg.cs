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
// Generated by IDLImporter from file nsIGenKeypairInfoDlg.idl
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
    /// nsIGeneratingKeypairInfoDialogs
    /// This is the interface for giving feedback to the user
    /// while generating a key pair.
    /// </summary>
    [ComImport()]
	[InterfaceType(ComInterfaceType.InterfaceIsIUnknown)]
	[Guid("11bf5cdc-1dd2-11b2-ba6a-c76afb326fa1")]
	public interface nsIGeneratingKeypairInfoDialogs
	{
		
		/// <summary>
        /// nsIGeneratingKeypairInfoDialogs
        /// This is the interface for giving feedback to the user
        /// while generating a key pair.
        /// </summary>
		[MethodImpl(MethodImplOptions.InternalCall, MethodCodeType=MethodCodeType.Runtime)]
		void DisplayGeneratingKeypairInfo([MarshalAs(UnmanagedType.Interface)] nsIInterfaceRequestor ctx, [MarshalAs(UnmanagedType.Interface)] nsIKeygenThread runnable);
	}
}
