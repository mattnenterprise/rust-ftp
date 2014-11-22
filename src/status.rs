
pub mod status {

	//1xx: Positive Preliminary Reply
	pub static STATUS_INITIATING: int      = 100;
	pub static STATUS_RESTART_MARKER: int  = 110;
	pub static STATUS_READY_MINUTE: int    = 120;
	pub static STATUS_ALREADY_OPEN: int    = 125;
	pub static STATUS_ABOUT_TO_SEND: int   = 150;

	//2xx: Positive Completion Reply
	pub static STATUS_COMMAND_OK: int               = 200;
	pub static STATUS_COMMAND_NOT_IMPLEMENTED: int  = 202;
	pub static STATUS_SYSTEM: int 				    = 211;
	pub static STATUS_DIRECTORY:int 				= 212;
	pub static STATUS_FILE: int 					= 213;
	pub static STATUS_HELP: int 					= 214;
	pub static STATUS_NAME: int 					= 215;
	pub static STATUS_READY: int 				    = 220;
	pub static STATUS_CLOSING: int 				    = 221;
	pub static STATUS_DATA_CONNECTION_OPEN: int 	= 225;
	pub static STATUS_CLOSING_DATA_CONNECTION: int  = 226;
	pub static STATUS_PASSIVE_MODE: int 			= 227;
	pub static STATUS_LONG_PASSIVE_MODE: int 		= 228;
	pub static STATUS_EETENDED_PASSIVE_MODE: int 	= 229;
	pub static STATUS_LOGGED_IN: int 				= 230;
	pub static STATUS_LOGGED_OUT: int 			    = 231;
	pub static STATUS_LOGOUT_ACK: int 			    = 232;
	pub static STATUS_REQUESTED_FILE_ACTION_OK: int = 250;
	pub static STATUS_PATH_CREATED: int 			= 257;

	//3xx: Positive Intermediate Reply
	pub static STATUS_USER_OK: int              = 331;
	pub static STATUS_LOGIN_NEED_ACCOUNT: int   = 332;
	pub static STATUS_REQUEST_FILE_PENDING: int = 350;

	//4xx: Transient Negative Completion Reply
	pub static STATUS_NOT_AVAILABLE: int 			   = 421;
	pub static STATUS_CANNOT_OPEN_DATA_Connection: int = 425;
	pub static STATUS_TRANSER_ABORTED: int            = 426;
	pub static STATUS_INVALID_CREDENTIALS: int 	       = 430;
	pub static STATUS_HOST_UNAVAILABLE: int            = 434;
	pub static STATUS_REQUEST_FILE_ACTION_IGNORED: int = 450;
	pub static STATUS_ACTION_ABORTED: int   		   = 451;
	pub static STATUS_REQUESTED_ACTION_NOT_TAKEN: int  = 452;	

	//5xx: Permanent Negative Completion Reply
	pub static STATUS_BAD_COMMAND: int     		      = 500;
	pub static STATUS_BAD_ARGUMENTS: int   		      = 501;
	pub static STATUS_NOT_IMPLEMENTED: int 		      = 502;
	pub static STATUS_BAD_SEQUENCE: int    		      = 503;
	pub static STATUS_NOT_IMPLEMENTED_PARAMETER: int  = 504;
	pub static STATUS_NOT_LOGGED_IN: int 			  = 530;
	pub static STATUS_STORING_NEEd_ACCOUNT: int       = 532;
	pub static STATUS_FILE_UNAVAILABLE: int 		  = 550;
	pub static STATUS_PAGE_TYPE_UNKNOWN: int          = 551;
	pub static STATUS_EXCEEDED_STORAGE: int           = 552;
	pub static STATUS_BAD_FILENAME: int               = 553;
}
